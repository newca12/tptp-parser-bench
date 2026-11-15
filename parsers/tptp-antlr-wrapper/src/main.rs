//! CLI wrapper for the ANTLR4-based TPTP parser (from playground-rs).
//!
//! Usage: tptp-antlr-wrapper <file.p>
//!
//! Reads the given TPTP file and parses the entire file using the ANTLR4
//! generated parser. Exits 0 on success, 1 on parse error.
//! Produces no output on success (designed for timing).

use std::cell::RefCell;
use std::env;
use std::fs;
use std::hint::black_box;
use std::process;
use std::rc::Rc;
use std::time::Instant;

use antlr4rust::error_listener::ErrorListener;
use antlr4rust::errors::ANTLRError;
use tptp_antlr::tptplexer::TPTPLexer;
use tptp_antlr::tptpparser::TPTPParser;

use antlr4rust::InputStream;
use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token_factory::CommonTokenFactory;
use antlr4rust::token_factory::TokenFactory;

#[derive(Clone, Default)]
struct SyntaxErrors(Rc<RefCell<Option<String>>>);

impl SyntaxErrors {
    fn first(&self) -> Option<String> {
        self.0.borrow().clone()
    }
}

impl<'input, T: Recognizer<'input>> ErrorListener<'input, T> for SyntaxErrors {
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<&<T::TF as TokenFactory<'input>>::Inner>,
        line: isize,
        column: isize,
        message: &str,
        _error: Option<&ANTLRError>,
    ) {
        let mut first = self.0.borrow_mut();
        if first.is_none() {
            *first = Some(format!("line {line}:{column} {message}"));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // --bench mode: in-process timed iterations
    if args.len() == 5 && args[1] == "--bench" {
        let warmup: usize = args[2].parse().unwrap_or(0);
        let iters: usize = args[3].parse().unwrap_or(50);
        bench_mode(warmup, iters, &args[4]);
        return;
    }

    let path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: tptp-antlr-wrapper <file.p>");
            process::exit(2);
        }
    };

    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(2);
        }
    };

    if let Err(error) = parse_antlr(&content) {
        eprintln!("parse error in {}: {}", path, error);
        process::exit(1);
    }
}

fn parse_antlr(content: &str) -> Result<(), String> {
    let tf = CommonTokenFactory;
    let errors = SyntaxErrors::default();
    let mut lexer = TPTPLexer::new_with_token_factory(InputStream::new(content), &tf);
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(errors.clone()));
    let token_source = CommonTokenStream::new(lexer);
    let mut parser = TPTPParser::new(token_source);
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(errors.clone()));
    let parse_result = parser.tptp_file();
    let syntax_error = errors.first();
    let parse_success = parse_result.is_ok();
    drop(parse_result);
    drop(parser);

    match (parse_success, syntax_error) {
        (true, None) => Ok(()),
        (_, Some(error)) => Err(error),
        (false, None) => Err("ANTLR parser rejected the input".to_string()),
    }
}

fn bench_mode(warmup: usize, iters: usize, path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(2);
        }
    };

    for _ in 0..warmup {
        if let Err(error) = parse_antlr(black_box(&content)) {
            eprintln!("parse error in {}: {}", path, error);
            process::exit(1);
        }
    }

    let mut times_ns = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        let result = parse_antlr(black_box(&content));
        let elapsed = start.elapsed().as_nanos();
        if let Err(error) = result {
            eprintln!("parse error in {}: {}", path, error);
            process::exit(1);
        }
        times_ns.push(elapsed);
    }

    println!(
        "BENCH_RESULTS_NS: [{}]",
        times_ns
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    if let Some(peak) = peak_memory_bytes() {
        println!("PEAK_MEMORY_BYTES: {}", peak);
    }
}

fn peak_memory_bytes() -> Option<u64> {
    let content = fs::read_to_string("/proc/self/status").ok()?;
    let line = content.lines().find(|line| line.starts_with("VmHWM:"))?;
    let kilobytes = line
        .strip_prefix("VmHWM:")?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    kilobytes.checked_mul(1024)
}

#[cfg(test)]
mod tests {
    use super::parse_antlr;

    #[test]
    fn rejects_syntax_errors_even_when_antlr_recovers_a_parse_tree() {
        assert!(parse_antlr("fof(bad,axiom,p &).").is_err());
    }

    #[test]
    fn accepts_a_syntactically_valid_formula() {
        assert!(parse_antlr("fof(good,axiom,p).").is_ok());
    }
}
