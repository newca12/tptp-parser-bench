//! CLI wrapper for the `mrs-tptp` crate (winnow-based parser).
//!
//! Usage: mrs-tptp-wrapper <file.p>
//!
//! Reads the given TPTP file and parses it. Exits 0 on success, 1 on
//! parse error. Produces no output on success (designed for timing).

use mrs_tptp::parse_tptp;
use std::env;
use std::fs;
use std::hint::black_box;
use std::process;
use std::time::Instant;

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
            eprintln!("usage: mrs-tptp-wrapper <file.p>");
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

    match parse_tptp(&content) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("parse error in {}: {}", path, e);
            process::exit(1);
        }
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
        match parse_tptp(black_box(&content)) {
            Ok(parsed) => {
                black_box(parsed);
            }
            Err(e) => {
                eprintln!("parse error in {}: {}", path, e);
                process::exit(1);
            }
        }
    }

    let mut times_ns = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        let res = parse_tptp(black_box(&content));
        let elapsed = start.elapsed().as_nanos();
        match res {
            Ok(parsed) => {
                black_box(parsed);
            }
            Err(e) => {
                eprintln!("parse error in {}: {}", path, e);
                process::exit(1);
            }
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
