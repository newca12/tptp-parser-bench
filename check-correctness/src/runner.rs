use crate::collect::collect_tptp_files;
use crate::output;
use crate::parsers::ParserBackend;
use crate::types::{CheckSummary, FileResult, ParseResult};

use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Instant;
use wait_timeout::ChildExt;

pub fn run(
    backend: &dyn ParserBackend,
    parser_id: &str,
    folder: &Path,
    timeout_ms: Option<u64>,
    threads: Option<usize>,
    verbose: bool,
) {
    let num_threads = threads.unwrap_or_else(|| std::cmp::min(rayon::current_num_threads(), 8));

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .stack_size(64 * 1024 * 1024)
        .build()
        .expect("Failed to build rayon thread pool");

    pool.install(|| run_inner(backend, parser_id, folder, timeout_ms, num_threads, verbose))
}

fn run_inner(
    backend: &dyn ParserBackend,
    parser_id: &str,
    folder: &Path,
    timeout_ms: Option<u64>,
    num_threads: usize,
    verbose: bool,
) {
    output::print_header(backend.name(), folder, timeout_ms);

    let files = match collect_tptp_files(folder) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("Failed to scan {}: {}", folder.display(), error);
            std::process::exit(2);
        }
    };
    if files.is_empty() {
        eprintln!("No .p files found in {}", folder.display());
        std::process::exit(2);
    }

    let total_files = files.len();
    output::print_found(total_files, num_threads);

    let progress = AtomicUsize::new(0);
    let start_time = Instant::now();

    let results: Vec<FileResult> = files
        .par_iter()
        .map(|file| {
            let relative_path = file
                .strip_prefix(folder)
                .unwrap_or(file)
                .display()
                .to_string();

            let result = if let Some(timeout) = timeout_ms {
                run_with_timeout(parser_id, file, timeout)
            } else {
                check_one(backend, file)
            };

            let current = progress.fetch_add(1, Ordering::Relaxed) + 1;
            output::print_progress(current, total_files, &relative_path, &result);

            FileResult {
                path: file.clone(),
                relative_path,
                result,
            }
        })
        .collect();

    let wall_time = start_time.elapsed();
    let summary = aggregate(&results, total_files, wall_time.as_secs_f64());
    output::print_summary(&summary, verbose);
}

pub fn check_one(backend: &dyn ParserBackend, file: &Path) -> ParseResult {
    let result = backend.check_file(file);
    if backend.verification_mode() == crate::parsers::VerificationMode::CrossValidate {
        cross_validate(result, file)
    } else {
        result
    }
}

fn run_with_timeout(parser_id: &str, file: &Path, timeout_ms: u64) -> ParseResult {
    use std::time::Duration;

    let start = Instant::now();
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return ParseResult::Failed {
                elapsed_ms: 0.0,
                error: format!("Failed to locate checker executable: {}", error),
            };
        }
    };
    let mut command = Command::new(executable);
    command
        .arg("--worker")
        .arg(parser_id)
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return ParseResult::Failed {
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                error: format!("Failed to spawn parser worker: {}", error),
            };
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (stdout_tx, stdout_rx) = mpsc::channel();
    let (stderr_tx, stderr_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stdout) = stdout {
            let _ = std::io::Read::read_to_end(&mut stdout, &mut bytes);
        }
        let _ = stdout_tx.send(bytes);
    });
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut stderr) = stderr {
            let _ = std::io::Read::read_to_end(&mut stderr, &mut bytes);
        }
        let _ = stderr_tx.send(bytes);
    });

    let status = match child.wait_timeout(Duration::from_millis(timeout_ms)) {
        Ok(Some(status)) => status,
        Ok(None) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            return ParseResult::Timeout { timeout_ms };
        }
        Err(error) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            return ParseResult::Failed {
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
                error: format!("Failed waiting for parser worker: {}", error),
            };
        }
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let remaining = Duration::from_millis(timeout_ms).saturating_sub(start.elapsed());
    let stdout = match stdout_rx.recv_timeout(remaining) {
        Ok(bytes) => bytes,
        Err(_) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            return ParseResult::Timeout { timeout_ms };
        }
    };
    let remaining = Duration::from_millis(timeout_ms).saturating_sub(start.elapsed());
    let stderr = match stderr_rx.recv_timeout(remaining) {
        Ok(bytes) => bytes,
        Err(_) => {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            return ParseResult::Timeout { timeout_ms };
        }
    };
    match status {
        status if status.success() => {
            let stdout = String::from_utf8_lossy(&stdout);
            match serde_json::from_str::<ParseResult>(stdout.trim()) {
                Ok(result) => result,
                Err(error) => ParseResult::Failed {
                    elapsed_ms,
                    error: format!("Invalid parser worker response: {}", error),
                },
            }
        }
        status => ParseResult::Failed {
            elapsed_ms,
            error: format!(
                "Parser worker exited with {}: {}",
                status,
                String::from_utf8_lossy(&stderr).trim()
            ),
        },
    }
}

pub fn set_elapsed(result: &mut ParseResult, elapsed_ms: f64) {
    match result {
        ParseResult::Success {
            elapsed_ms: value, ..
        }
        | ParseResult::RoundtripFail {
            elapsed_ms: value, ..
        }
        | ParseResult::Failed {
            elapsed_ms: value, ..
        } => *value = elapsed_ms,
        ParseResult::ReadError { .. } | ParseResult::Timeout { .. } => {}
    }
}

/// Cross-validate formula count against mrs-tptp reference
fn cross_validate(result: ParseResult, file: &Path) -> ParseResult {
    use crate::parsers::reference;

    match &result {
        ParseResult::Success {
            formula_count,
            include_count,
            elapsed_ms,
        } => match reference::parse_file(file) {
            Ok((ref_formulas, ref_includes)) => {
                if *formula_count != ref_formulas {
                    ParseResult::RoundtripFail {
                        formula_count: *formula_count,
                        elapsed_ms: *elapsed_ms,
                        error: format!(
                            "Formula count mismatch: parser={}, mrs-tptp reference={}",
                            formula_count, ref_formulas
                        ),
                    }
                } else if *include_count != ref_includes {
                    ParseResult::RoundtripFail {
                        formula_count: *formula_count,
                        elapsed_ms: *elapsed_ms,
                        error: format!(
                            "Include count mismatch: parser={}, mrs-tptp reference={}",
                            include_count, ref_includes
                        ),
                    }
                } else {
                    result
                }
            }
            Err(error) => ParseResult::RoundtripFail {
                formula_count: *formula_count,
                elapsed_ms: *elapsed_ms,
                error: format!("Reference parser could not validate this file: {}", error),
            },
        },
        _ => result,
    }
}

fn aggregate(results: &[FileResult], total_files: usize, wall_time_secs: f64) -> CheckSummary {
    let mut summary = CheckSummary {
        total_files,
        wall_time_secs,
        ..Default::default()
    };

    for fr in results {
        match &fr.result {
            ParseResult::Success {
                formula_count,
                include_count,
                elapsed_ms,
            } => {
                summary.successful += 1;
                summary.total_formulas += formula_count;
                summary.total_includes += include_count;
                summary.total_parse_time_ms += elapsed_ms;
            }
            ParseResult::RoundtripFail {
                formula_count,
                elapsed_ms,
                error,
            } => {
                summary.roundtrip_failures += 1;
                summary.total_formulas += formula_count;
                summary.total_parse_time_ms += elapsed_ms;
                summary.roundtrip_failed_files.push((
                    fr.path.clone(),
                    fr.relative_path.clone(),
                    error.clone(),
                ));
            }
            ParseResult::Failed { elapsed_ms, error } => {
                summary.failed += 1;
                summary.total_parse_time_ms += elapsed_ms;
                summary.failed_files.push((fr.path.clone(), error.clone()));
            }
            ParseResult::ReadError { error } => {
                summary.failed += 1;
                summary.failed_files.push((fr.path.clone(), error.clone()));
            }
            ParseResult::Timeout { timeout_ms } => {
                summary.timeouts += 1;
                summary.timeout_files.push((fr.path.clone(), *timeout_ms));
            }
        }
    }

    summary
}
