use crate::types::{CheckSummary, ParseResult};
use std::path::Path;

pub fn print_header(parser_name: &str, folder: &Path, timeout_ms: Option<u64>) {
    println!("###################################");
    println!("TPTP Parser Test Suite (Parallel)");
    println!("###################################");
    println!();
    println!("Parser: {}", parser_name);
    println!("Scanning folder: {}", folder.display());
    if let Some(timeout) = timeout_ms {
        println!("Timeout per file: {}ms", timeout);
    }
    println!();
}

pub fn print_found(total_files: usize, num_threads: usize) {
    println!("Found {} TPTP files to parse", total_files);
    println!("Using {} threads", num_threads);
    println!();
    println!("{}", "=".repeat(70));
}

pub fn print_progress(current: usize, total: usize, relative_path: &str, result: &ParseResult) {
    let status = match result {
        ParseResult::Success {
            formula_count,
            include_count,
            elapsed_ms,
        } => format!(
            "OK ({:.2}ms, {} formulas, {} includes)",
            elapsed_ms,
            formula_count.map_or_else(|| "unknown".to_string(), |count| count.to_string()),
            include_count.map_or_else(|| "unknown".to_string(), |count| count.to_string())
        ),
        ParseResult::RoundtripFail {
            formula_count,
            elapsed_ms,
            ..
        } => format!(
            "ROUNDTRIP FAIL ({:.2}ms, {} formulas)",
            elapsed_ms,
            formula_count.map_or_else(|| "unknown".to_string(), |count| count.to_string())
        ),
        ParseResult::Failed { elapsed_ms, error } => {
            format!("FAILED ({:.2}ms)\n        Error: {}", elapsed_ms, error)
        }
        ParseResult::ReadError { error } => format!("FAILED ({})", error),
        ParseResult::Timeout { timeout_ms } => format!("TIMEOUT (>{}ms)", timeout_ms),
    };

    println!(
        "[{:>4}/{}] Parsing {} ... {}",
        current, total, relative_path, status
    );
}

pub fn print_summary(summary: &CheckSummary, verbose: bool) {
    println!();
    println!("{}", "=".repeat(70));
    println!();
    println!("###################################");
    println!("Summary");
    println!("###################################");
    println!();
    println!("Total files:      {}", summary.total_files);
    println!(
        "Verified:         {} ({:.1}%)",
        summary.successful,
        if summary.total_files > 0 {
            (summary.successful as f64 / summary.total_files as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "Failed:           {} ({:.1}%)",
        summary.failed,
        if summary.total_files > 0 {
            (summary.failed as f64 / summary.total_files as f64) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "Timeouts:         {} ({:.1}%)",
        summary.timeouts,
        if summary.total_files > 0 {
            (summary.timeouts as f64 / summary.total_files as f64) * 100.0
        } else {
            0.0
        }
    );
    println!();
    if summary.files_with_known_formula_count == 0 {
        println!("Total formulas:   unknown");
    } else {
        println!(
            "Total formulas:   {} (known for {} of {} parsed files)",
            summary.total_formulas,
            summary.files_with_known_formula_count,
            summary.successful + summary.roundtrip_failures
        );
    }
    if summary.files_with_known_include_count == 0 {
        println!("Total includes:   unknown");
    } else {
        println!(
            "Total includes:   {} (known for {} of {} successful files)",
            summary.total_includes, summary.files_with_known_include_count, summary.successful
        );
    }
    println!("Roundtrip fails:  {}", summary.roundtrip_failures);
    println!();
    println!("Wall time:        {:.2}s", summary.wall_time_secs);
    println!("CPU time (sum):   {:.2}ms", summary.total_parse_time_ms);

    if summary.total_files > 0 {
        println!(
            "Avg time/file:    {:.2}ms",
            summary.total_parse_time_ms / summary.total_files as f64
        );
        println!(
            "Throughput:       {:.1} files/sec",
            summary.total_files as f64 / summary.wall_time_secs
        );
    }

    if !summary.failed_files.is_empty() {
        println!();
        println!("###################################");
        println!("Parse Failed Files");
        println!("###################################");
        println!();
        for (file, error) in &summary.failed_files {
            println!("  {} ", file.display());
            println!("    Error: {}", error);
        }
    }

    if !summary.roundtrip_failed_files.is_empty() {
        println!();
        println!("###################################");
        println!("Roundtrip Failed Files");
        println!("###################################");
        println!();
        for (file, _rel_path, error) in &summary.roundtrip_failed_files {
            println!("  {} ", file.display());
            println!("    {}", error);
            println!();
        }
    }

    if !summary.timeout_files.is_empty() && verbose {
        println!();
        println!("###################################");
        println!("Timed Out Files");
        println!("###################################");
        println!();
        for (file, timeout) in &summary.timeout_files {
            println!("  {}  (>{}ms)", file.display(), timeout);
        }
    }

    if !summary.failed_files.is_empty()
        || !summary.roundtrip_failed_files.is_empty()
        || !summary.timeout_files.is_empty()
    {
        std::process::exit(1);
    } else {
        println!();
        println!("\u{2713} All files parsed and verified successfully!");
    }
}
