//! CLI wrapper for the `tptp` crate (nom-based parser by Michael Rawson).
//!
//! Usage: tptp-nom-wrapper <file.p>
//!
//! Reads the given TPTP file and parses it. Exits 0 on success, 1 on
//! parse error. Produces no output on success (designed for timing).

use std::env;
use std::fs;
use std::hint::black_box;
use std::process;
use std::time::Instant;
use tptp::TPTPIterator;

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
            eprintln!("usage: tptp-nom-wrapper <file.p>");
            process::exit(2);
        }
    };

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(2);
        }
    };

    if !parse_nom(&bytes) {
        eprintln!("syntax or trailing error in {}", path);
        process::exit(1);
    }
}

fn parse_nom(bytes: &[u8]) -> bool {
    let mut parser = TPTPIterator::<()>::new(bytes);
    for input in &mut parser {
        if input.is_err() {
            return false;
        }
    }
    parser.remaining.is_empty()
}

fn bench_mode(warmup: usize, iters: usize, path: &str) {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(2);
        }
    };

    for _ in 0..warmup {
        if !parse_nom(black_box(&bytes)) {
            eprintln!("syntax or trailing error in {}", path);
            process::exit(1);
        }
    }

    let mut times_ns = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        let ok = parse_nom(black_box(&bytes));
        let elapsed = start.elapsed().as_nanos();
        if !ok {
            eprintln!("syntax or trailing error in {}", path);
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
