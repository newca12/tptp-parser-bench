use crate::config::{InputMode, ParserConfig, Settings};
use anyhow::Context;
use serde::Serialize;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
pub fn run_benchmark(
    parser: &ParserConfig,
    file_path: &Path,
    settings: &Settings,
) -> BenchResult {
    let file_name = file_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
    let file_content = std::fs::read(file_path).ok();

    // Check that the parser command exists before running iterations.
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
) -> anyhow::Result<Duration> {
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

    let start = Instant::now();
    let mut child = cmd.spawn().context("failed to spawn parser process")?;

    // Pipe file content to stdin if needed.
    if let InputMode::Stdin = parser.input_mode {
        if let (Some(mut stdin), Some(content)) = (child.stdin.take(), file_content) {
            stdin.write_all(content)?;
            // Drop stdin to signal EOF.
            drop(stdin);
        }
    }

    let output = child
        .wait_with_output()
        .context("failed to wait for parser")?;
    let elapsed = start.elapsed();

    if elapsed > Duration::from_secs(timeout_secs) {
        anyhow::bail!("timeout after {}s", timeout_secs);
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "parser exited with {}: {}",
            output.status,
            stderr.trim()
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
    let variance = times_ns.iter().map(|&t| (t as f64 - mean).powi(2)).sum::<f64>() / n;
    (mean, variance.sqrt())
}

/// Check that a command exists (either an absolute/relative path or in PATH).
fn which_or_exists(cmd: &str) -> Result<(), String> {
    let path = Path::new(cmd);
    if path.is_absolute() || cmd.starts_with("./") || cmd.starts_with("../") {
        if path.exists() {
            Ok(())
        } else {
            Err(format!("command not found: {}", cmd))
        }
    } else {
        // Search PATH
        match Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) if status.success() => Ok(()),
            _ => Err(format!("command not found in PATH: {}", cmd)),
        }
    }
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
