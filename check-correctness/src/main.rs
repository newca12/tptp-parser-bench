mod collect;
mod output;
mod parsers;
mod runner;
mod types;

use clap::Parser;
use std::path::PathBuf;

/// TPTP Parser Correctness Checker
///
/// Recursively parse all .p files in a folder with a specified parser,
/// performing roundtrip or cross-validation correctness checks.
#[derive(Parser)]
#[command(name = "check-correctness", version, about)]
struct Cli {
    /// Parser to check
    #[arg(value_enum)]
    parser: ParserChoice,

    /// Folder containing .p files (searched recursively)
    folder: PathBuf,

    /// Timeout per file in milliseconds
    #[arg(long)]
    timeout: Option<u64>,

    /// Number of threads (default: min(system_threads, 8))
    #[arg(long)]
    threads: Option<usize>,

    /// Show detailed output for timed out files
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Clone, clap::ValueEnum)]
enum ParserChoice {
    MrsTptp,
    TptpNom,
    TptpAntlr,
    VampireTptp,
    ScalaTptp,
    JavaAntlr,
}

fn main() {
    let cli = Cli::parse();

    let backend: Box<dyn parsers::ParserBackend> = match cli.parser {
        ParserChoice::MrsTptp => Box::new(parsers::mrs_tptp::MrsTptpBackend),
        ParserChoice::TptpNom => Box::new(parsers::tptp_nom::TptpNomBackend),
        ParserChoice::TptpAntlr => Box::new(parsers::tptp_antlr::TptpAntlrBackend),
        ParserChoice::VampireTptp => match parsers::vampire::VampireBackend::new() {
            Ok(b) => Box::new(b),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            }
        },
        ParserChoice::ScalaTptp => match parsers::scala_tptp::ScalaTptpBackend::new() {
            Ok(b) => Box::new(b),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            }
        },
        ParserChoice::JavaAntlr => match parsers::java_antlr::JavaAntlrBackend::new() {
            Ok(b) => Box::new(b),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(2);
            }
        },
    };

    runner::run(
        backend.as_ref(),
        &cli.folder,
        cli.timeout,
        cli.threads,
        cli.verbose,
    );
}
