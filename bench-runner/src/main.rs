mod config;
mod report;
mod runner;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

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
        return Ok(());
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
        eprintln!(
            "⚠  No .p or .tptp files found in {}",
            cli.corpus.display()
        );
        return Ok(());
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
                println!(
                    "  ✓ {} : {:.2} ms ± {:.2} ms ({:.2} MB/s)",
                    result.file_name,
                    result.mean_ns / 1e6,
                    result.stddev_ns / 1e6,
                    result.throughput_mb_s,
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
    Ok(())
}
