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
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--worker") {
        if args.len() != 4 {
            eprintln!("invalid worker invocation");
            std::process::exit(2);
        }
        let backend = match create_backend(&args[2]) {
            Ok(backend) => backend,
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(2);
            }
        };
        let start = std::time::Instant::now();
        let mut result = runner::check_one(backend.as_ref(), std::path::Path::new(&args[3]));
        runner::set_elapsed(&mut result, start.elapsed().as_secs_f64() * 1000.0);
        match serde_json::to_string(&result) {
            Ok(json) => println!("{}", json),
            Err(error) => {
                eprintln!("failed to serialize worker result: {}", error);
                std::process::exit(2);
            }
        }
        return;
    }

    let cli = Cli::parse();

    let parser_id = parser_id(&cli.parser);
    if cli.threads == Some(0) {
        eprintln!("--threads must be greater than zero");
        std::process::exit(2);
    }
    let backend = match create_backend(parser_id) {
        Ok(backend) => backend,
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(2);
        }
    };

    runner::run(
        backend.as_ref(),
        parser_id,
        &cli.folder,
        cli.timeout,
        cli.threads,
        cli.verbose,
    );
}

fn parser_id(choice: &ParserChoice) -> &'static str {
    match choice {
        ParserChoice::MrsTptp => "mrs-tptp",
        ParserChoice::TptpNom => "tptp-nom",
        ParserChoice::TptpAntlr => "tptp-antlr",
        ParserChoice::VampireTptp => "vampire-tptp",
        ParserChoice::ScalaTptp => "scala-tptp",
        ParserChoice::JavaAntlr => "java-antlr",
    }
}

fn create_backend(parser_id: &str) -> Result<Box<dyn parsers::ParserBackend>, String> {
    match parser_id {
        "mrs-tptp" => Ok(Box::new(parsers::mrs_tptp::MrsTptpBackend)),
        "tptp-nom" => Ok(Box::new(parsers::tptp_nom::TptpNomBackend)),
        "tptp-antlr" => Ok(Box::new(parsers::tptp_antlr::TptpAntlrBackend)),
        "vampire-tptp" => Ok(Box::new(parsers::vampire::VampireBackend::new()?)),
        "scala-tptp" => Ok(Box::new(parsers::scala_tptp::ScalaTptpBackend::new()?)),
        "java-antlr" => Ok(Box::new(parsers::java_antlr::JavaAntlrBackend::new()?)),
        _ => Err(format!("unknown parser: {}", parser_id)),
    }
}
