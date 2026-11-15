//! CLI wrapper for Vampire's TPTP parser.
//!
//! Usage: vampire-tptp-wrapper <file.p>
//!        vampire-tptp-wrapper --check <file.p>
//!
//! Reads the given TPTP file and parses it using Vampire's native
//! TPTP parser (Parse::TPTP). Exits 0 on success, 1 on parse error.
//! Produces no output on success (designed for timing).
//!
//! In --check mode, reports unknown counts: Vampire's internal unit count is
//! not comparable to source-level formula counts across TPTP dialects.

use std::env;
use std::ffi::CString;
use std::process;

unsafe extern "C" {
    fn parse_tptp_file(path: *const std::os::raw::c_char) -> std::os::raw::c_int;
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

    // --check mode: output formula count to stdout
    if args.len() == 3 && args[1] == "--check" {
        check_mode(&args[2]);
        return;
    }

    // Benchmark mode: silent on success
    let path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: vampire-tptp-wrapper <file.p>");
            process::exit(2);
        }
    };

    let c_path = match CString::new(path.as_bytes()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("invalid path: contains null byte");
            process::exit(2);
        }
    };

    let result = unsafe { parse_tptp_file(c_path.as_ptr()) };

    match result {
        0 => {} // success, silent exit
        1 => {
            eprintln!("parse error in {}", path);
            process::exit(1);
        }
        _ => {
            eprintln!("error reading {}", path);
            process::exit(2);
        }
    }
}

fn check_mode(path: &str) {
    let c_path = match CString::new(path.as_bytes()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("invalid path: contains null byte");
            process::exit(2);
        }
    };

    let result = unsafe { parse_tptp_file(c_path.as_ptr()) };

    match result {
        0 => {
            println!("OK formulas=unknown includes=unknown");
        }
        1 => {
            eprintln!("parse error in {}", path);
            process::exit(1);
        }
        _ => {
            eprintln!("error reading {}", path);
            process::exit(2);
        }
    }
}

fn bench_mode(warmup: usize, iters: usize, path: &str) {
    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| std::path::PathBuf::from(path));
    let c_path = match CString::new(abs_path.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("invalid path: contains null byte");
            process::exit(2);
        }
    };

    for _ in 0..warmup {
        let result = unsafe { parse_tptp_file(c_path.as_ptr()) };
        if result != 0 {
            eprintln!("parse or read error in {}", path);
            process::exit(1);
        }
    }

    let mut times_ns = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = std::time::Instant::now();
        let result = unsafe { parse_tptp_file(c_path.as_ptr()) };
        let elapsed = start.elapsed().as_nanos();
        if result != 0 {
            eprintln!("parse or read error in {}", path);
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
}
