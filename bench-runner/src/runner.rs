use crate::config::{InputMode, ParserConfig, Settings};
use serde::Serialize;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

/// Result of benchmarking one parser on one file.
#[derive(Debug, Serialize, Clone)]
pub struct BenchResult {
    pub parser_name: String,
    pub display_name: String,
    pub language: String,
    pub technique: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub times_ns: Vec<u128>,
    pub mean_ns: f64,
    pub stddev_ns: f64,
    pub throughput_mb_s: f64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Run the benchmark: warmup + timed iterations.
pub fn run_benchmark(parser: &ParserConfig, file_path: &Path, settings: &Settings) -> BenchResult {
    let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
    let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
    let file_content = match &parser.input_mode {
        InputMode::Stdin => match std::fs::read(file_path) {
            Ok(content) => Some(content),
            Err(error) => {
                return make_error_result(
                    parser,
                    &file_name,
                    file_size,
                    &format!("failed to read input file: {error}"),
                );
            }
        },
        InputMode::FileArg => None,
    };

    // Check that the parser command exists before running iterations.
    if parser.run_cmd.is_empty() {
        return make_error_result(parser, &file_name, file_size, "run_cmd must not be empty");
    }
    if let Err(e) = which_or_exists(&parser.run_cmd[0]) {
        return make_error_result(parser, &file_name, file_size, &e);
    }

    let mut times_ns = Vec::with_capacity(settings.iterations);
    let total_iters = settings.warmup_iterations + settings.iterations;

    for i in 0..total_iters {
        let is_warmup = i < settings.warmup_iterations;
        match run_once(
            parser,
            file_path,
            file_content.as_deref(),
            settings.timeout_secs,
        ) {
            Ok(duration) => {
                if !is_warmup {
                    times_ns.push(duration.as_nanos());
                }
            }
            Err(e) => {
                return make_error_result(
                    parser,
                    &file_name,
                    file_size,
                    &format!(
                        "failed on iteration {} ({}): {}",
                        i + 1,
                        if is_warmup { "warmup" } else { "timed" },
                        e
                    ),
                );
            }
        }
    }

    let (mean_ns, stddev_ns) = stats(&times_ns);
    let mean_secs = mean_ns / 1e9;
    let throughput_mb_s = if mean_secs > 0.0 {
        (file_size as f64 / 1e6) / mean_secs
    } else {
        0.0
    };

    BenchResult {
        parser_name: parser.name.clone(),
        display_name: parser.display_name.clone(),
        language: parser.language.clone(),
        technique: parser.technique.clone(),
        file_name,
        file_size_bytes: file_size,
        times_ns,
        mean_ns,
        stddev_ns,
        throughput_mb_s,
        success: true,
        error: None,
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Execute the parser once on the given file and return wall-clock duration.
fn run_once(
    parser: &ParserConfig,
    file_path: &Path,
    file_content: Option<&[u8]>,
    timeout_secs: u64,
) -> crate::Result<Duration> {
    let program = &parser.run_cmd[0];
    let args = &parser.run_cmd[1..];

    let mut cmd = Command::new(program);
    cmd.args(args);

    match parser.input_mode {
        InputMode::FileArg => {
            cmd.arg(file_path);
            cmd.stdin(Stdio::null());
        }
        InputMode::Stdin => {
            cmd.stdin(Stdio::piped());
        }
    }
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let start = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|error| io::Error::other(format!("failed to spawn parser process: {error}")))?;
    let stderr_reader = child.stderr.take().map(|mut stderr| {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = std::io::Read::read_to_end(&mut stderr, &mut bytes);
            let _ = sender.send(bytes);
        });
        receiver
    });

    // Pipe file content to stdin if needed.
    let stdin_writer = if let InputMode::Stdin = parser.input_mode {
        let stdin = child.stdin.take();
        let content = file_content.map(ToOwned::to_owned);
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| -> std::io::Result<()> {
                if let (Some(mut stdin), Some(content)) = (stdin, content) {
                    stdin.write_all(&content)?;
                }
                Ok(())
            })();
            let _ = sender.send(result);
        });
        Some(receiver)
    } else {
        None
    };

    let status = match child.wait_timeout(Duration::from_secs(timeout_secs)) {
        Err(error) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                io::Error::other(format!("failed waiting for parser process: {error}")).into(),
            );
        }
        Ok(Some(status)) => status,
        Ok(None) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timeout after {}s", timeout_secs),
            )
            .into());
        }
    };

    if let Some(writer) = stdin_writer {
        let remaining = Duration::from_secs(timeout_secs).saturating_sub(start.elapsed());
        match writer.recv_timeout(remaining) {
            Ok(result) => result?,
            Err(_) => {
                #[cfg(unix)]
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                let _ = child.kill();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("timeout after {}s", timeout_secs),
                )
                .into());
            }
        }
    }

    let stderr = if let Some(reader) = stderr_reader {
        let remaining = Duration::from_secs(timeout_secs).saturating_sub(start.elapsed());
        match reader.recv_timeout(remaining) {
            Ok(stderr) => stderr,
            Err(_) => {
                #[cfg(unix)]
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                let _ = child.kill();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("timeout after {}s", timeout_secs),
                )
                .into());
            }
        }
    } else {
        Vec::new()
    };
    let elapsed = start.elapsed();

    if elapsed > Duration::from_secs(timeout_secs) {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            format!("timeout after {}s", timeout_secs),
        )
        .into());
    }

    if !status.success() {
        let stderr = String::from_utf8_lossy(&stderr);
        return Err(
            io::Error::other(format!("parser exited with {}: {}", status, stderr.trim())).into(),
        );
    }

    Ok(elapsed)
}

/// Mean and standard deviation of nanosecond timings.
fn stats(times_ns: &[u128]) -> (f64, f64) {
    if times_ns.is_empty() {
        return (0.0, 0.0);
    }
    let n = times_ns.len() as f64;
    let mean = times_ns.iter().sum::<u128>() as f64 / n;
    let variance = times_ns
        .iter()
        .map(|&t| (t as f64 - mean).powi(2))
        .sum::<f64>()
        / n;
    (mean, variance.sqrt())
}

/// Check that a command exists (either an absolute/relative path or in PATH).
fn which_or_exists(cmd: &str) -> Result<(), String> {
    let path = Path::new(cmd);
    if path.is_absolute() || cmd.contains('/') || cmd.contains('\\') {
        if is_executable_file(path) {
            Ok(())
        } else {
            Err(format!("command not found: {}", cmd))
        }
    } else {
        let path_var = std::env::var_os("PATH").unwrap_or_default();
        for directory in std::env::split_paths(&path_var) {
            if command_in_directory(&directory, cmd) {
                return Ok(());
            }
        }
        Err(format!("command not found in PATH: {}", cmd))
    }
}

fn command_in_directory(directory: &Path, cmd: &str) -> bool {
    let candidate = directory.join(cmd);
    if is_executable_file(&candidate) {
        return true;
    }

    #[cfg(windows)]
    if candidate.extension().is_none() {
        let extensions =
            std::env::var_os("PATHEXT").unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".into());
        return extensions.to_string_lossy().split(';').any(|extension| {
            is_executable_file(&candidate.with_extension(extension.trim_start_matches('.')))
        });
    }

    false
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn make_error_result(
    parser: &ParserConfig,
    file_name: &str,
    file_size: u64,
    error: &str,
) -> BenchResult {
    BenchResult {
        parser_name: parser.name.clone(),
        display_name: parser.display_name.clone(),
        language: parser.language.clone(),
        technique: parser.technique.clone(),
        file_name: file_name.to_string(),
        file_size_bytes: file_size,
        times_ns: vec![],
        mean_ns: 0.0,
        stddev_ns: 0.0,
        throughput_mb_s: 0.0,
        success: false,
        error: Some(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_executable_file, which_or_exists};

    #[test]
    fn resolves_the_current_executable_without_external_which() {
        let current_exe = std::env::current_exe().expect("test executable path");
        assert!(is_executable_file(&current_exe));
        assert!(which_or_exists(current_exe.to_str().expect("UTF-8 executable path")).is_ok());
    }

    #[test]
    fn resolves_commands_from_path() {
        #[cfg(windows)]
        let command = "cmd";
        #[cfg(not(windows))]
        let command = "sh";

        assert!(which_or_exists(command).is_ok());
    }

    #[test]
    fn reports_missing_commands() {
        assert!(which_or_exists("tptp-parser-bench-command-that-does-not-exist").is_err());
    }
}
