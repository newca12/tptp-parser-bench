//! CLI wrapper for the ANTLR4-based TPTP parser (from playground-rs).
//!
//! Usage: tptp-antlr-wrapper <file.p>
//!
//! Reads the given TPTP file and parses the entire file using the ANTLR4
//! generated parser. Exits 0 on success, 1 on parse error.
//! Produces no output on success (designed for timing).

use std::env;
use std::fs;
use std::hint::black_box;
use std::process;
use std::time::Instant;

use tptp_antlr::tptplexer::TPTPLexer;
use tptp_antlr::tptpparser::TPTPParser;

use antlr4rust::InputStream;
use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::token_factory::CommonTokenFactory;

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

    if !parse_antlr(&content) {
        eprintln!("parse error in {}", path);
        process::exit(1);
    }
}

fn parse_antlr(content: &str) -> bool {
    let tf = CommonTokenFactory;
    let lexer = TPTPLexer::new_with_token_factory(InputStream::new(content), &tf);
    let token_source = CommonTokenStream::new(lexer);
    let mut parser = TPTPParser::new(token_source);
    let success = parser.tptp_file().is_ok();
    drop(parser);
    success
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
        if !parse_antlr(black_box(&content)) {
            eprintln!("parse error in {}", path);
            process::exit(1);
        }
    }

    let mut times_ns = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        let ok = parse_antlr(black_box(&content));
        let elapsed = start.elapsed().as_nanos();
        if !ok {
            eprintln!("parse error in {}", path);
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
    let mut hwm = None;
    let mut peak = None;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            if let Some(val) = rest.split_whitespace().next() {
                if let Ok(kb) = val.parse::<u64>() {
                    hwm = Some(kb * 1024);
                }
            }
        } else if let Some(rest) = line.strip_prefix("VmPeak:") {
            if let Some(val) = rest.split_whitespace().next() {
                if let Ok(kb) = val.parse::<u64>() {
                    peak = Some(kb * 1024);
                }
            }
        }
    }
    hwm.or(peak)
}
