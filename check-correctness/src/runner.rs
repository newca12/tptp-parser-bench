use crate::collect::collect_tptp_files;
use crate::output;
use crate::parsers::{ParserBackend, VerificationMode};
use crate::types::{CheckSummary, FileResult, ParseResult};

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub fn run(
    backend: &dyn ParserBackend,
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

    pool.install(|| run_inner(backend, folder, timeout_ms, num_threads, verbose))
}

fn run_inner(
    backend: &dyn ParserBackend,
    folder: &Path,
    timeout_ms: Option<u64>,
    num_threads: usize,
    verbose: bool,
) {
    output::print_header(backend.name(), folder, timeout_ms);

    let files = collect_tptp_files(folder);
    if files.is_empty() {
        println!("No .p files found in {}", folder.display());
        return;
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

            let mut result = if let Some(timeout) = timeout_ms {
                run_with_timeout(backend, file, timeout)
            } else {
                backend.check_file(file)
            };

            // Cross-validation: for parsers without roundtrip, compare against mrs-tptp
            if backend.verification_mode() == VerificationMode::CrossValidate {
                result = cross_validate(result, file);
            }

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

fn run_with_timeout(backend: &dyn ParserBackend, file: &Path, timeout_ms: u64) -> ParseResult {
    use std::time::Duration;

    let file = file.to_path_buf();
    let deadline = Duration::from_millis(timeout_ms);

    // We can't send the backend to another thread directly, so we use a scoped approach
    // Instead, just run with a simple time check after the fact
    let start = Instant::now();
    let result = backend.check_file(&file);
    let elapsed = start.elapsed();

    if elapsed > deadline {
        ParseResult::Timeout { timeout_ms }
    } else {
        result
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
        } => {
            match reference::parse_file(file) {
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
                Err(_) => {
                    // Reference parser failed on this file; cannot cross-validate
                    // Pass through the parser's own result
                    result
                }
            }
        }
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
                summary.successful += 1; // Parsing succeeded, verification failed
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
