mod config;
mod report;
mod runner;

use clap::Parser;
use std::error::Error;
use std::io;
use std::path::PathBuf;

pub type BenchError = Box<dyn Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BenchError>;

/// 🏟️  TPTP Parser Benchmarking Arena
///
/// Run TPTP parsers on a shared corpus and generate comparison reports.
#[derive(Parser)]
#[command(name = "bench-runner", version, about)]
struct Cli {
    /// Path to the parser configuration file.
    #[arg(short, long, default_value = "parsers.toml")]
    config: PathBuf,

    /// Directory containing .p / .tptp corpus files.
    #[arg(short = 'd', long, default_value = "corpus")]
    corpus: PathBuf,

    /// Output directory for results and reports.
    #[arg(short, long, default_value = "results")]
    output: PathBuf,

    /// Only write JSON output (skip HTML report).
    #[arg(long)]
    json_only: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config.
    let config = config::Config::load(&cli.config)?;
    let enabled = config.enabled_parsers();

    if enabled.is_empty() {
        eprintln!("⚠  No enabled parsers found in {}", cli.config.display());
        eprintln!("   Edit parsers.toml and set enabled = true for at least one parser.");
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "no enabled parsers configured").into(),
        );
    }

    // Collect corpus files.
    let mut corpus_files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&cli.corpus)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map_or(false, |ext| ext == "p" || ext == "tptp")
        {
            corpus_files.push(path);
        }
    }
    corpus_files.sort();

    if corpus_files.is_empty() {
        eprintln!("⚠  No .p or .tptp files found in {}", cli.corpus.display());
        return Err(io::Error::new(io::ErrorKind::NotFound, "no TPTP corpus files found").into());
    }

    // Print banner.
    println!("🏟️  TPTP Parser Arena");
    println!("   Parsers : {}", enabled.len());
    println!("   Files   : {}", corpus_files.len());
    println!(
        "   Iters   : {} (+ {} warmup)",
        config.settings.iterations, config.settings.warmup_iterations
    );
    println!();

    // Run benchmarks.
    let mut all_results: Vec<runner::BenchResult> = Vec::new();

    for parser in &enabled {
        println!("▶ {}", parser.display_name);

        for file in &corpus_files {
            let result = runner::run_benchmark(parser, file, &config.settings);

            if result.success {
                let mem_str = result
                    .peak_memory_bytes
                    .map(|b| format!(", peak mem: {}", format_bytes(b)))
                    .unwrap_or_default();
                println!(
                    "  ✓ {} : {} (median), {} ± {} ({}{})",
                    result.file_name,
                    format_time_ns(result.median_ns),
                    format_time_ns(result.mean_ns),
                    format_time_ns(result.stddev_ns),
                    format_throughput(result.throughput_mb_s),
                    mem_str,
                );
            } else {
                println!(
                    "  ✗ {} : {}",
                    result.file_name,
                    result.error.as_deref().unwrap_or("unknown error"),
                );
            }

            all_results.push(result);
        }
        println!();
    }

    // Create output directory.
    std::fs::create_dir_all(&cli.output)?;

    // Write raw results JSON.
    let json_path = cli.output.join("results.json");
    let json = serde_json::to_string_pretty(&all_results)?;
    std::fs::write(&json_path, &json)?;
    println!("📄 {}", json_path.display());

    // Write GitHub Action compatible JSON.
    let gh_json = report::generate_github_action_json(&all_results);
    let gh_path = cli.output.join("gh-bench.json");
    std::fs::write(&gh_path, &gh_json)?;
    println!("📄 {}", gh_path.display());

    // Generate HTML report.
    if !cli.json_only {
        let html_path = cli.output.join("report.html");
        report::generate_html_report(&all_results, &html_path)?;
        println!("📊 {}", html_path.display());
    }

    println!("\n✅ Done.");
    let failed = all_results.iter().filter(|result| !result.success).count();
    if failed > 0 {
        return Err(io::Error::other(format!(
            "{} benchmark case(s) failed; see the results above and report",
            failed
        ))
        .into());
    }
    Ok(())
}

/// Format throughput with adaptive units (MB/s or KB/s).
fn format_throughput(mb_s: f64) -> String {
    if mb_s >= 0.1 {
        format!("{:.2} MB/s", mb_s)
    } else {
        format!("{:.2} KB/s", mb_s * 1000.0)
    }
}

/// Format nanosecond timings with adaptive units (ns, µs, ms, s).
fn format_time_ns(ns: f64) -> String {
    if ns >= 1e9 {
        format!("{:.2} s", ns / 1e9)
    } else if ns >= 1e6 {
        format!("{:.2} ms", ns / 1e6)
    } else if ns >= 1e3 {
        format!("{:.2} µs", ns / 1e3)
    } else {
        format!("{:.0} ns", ns)
    }
}

/// Format bytes with adaptive binary units (B, KB, MB, GB).
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
