//! CLI wrapper for Vampire's TPTP parser.
//!
//! Usage: vampire-tptp-wrapper <file.p>
//!        vampire-tptp-wrapper --check <file.p>
//!
//! Reads the given TPTP file and parses it using Vampire's native
//! TPTP parser (Parse::TPTP). Exits 0 on success, 1 on parse error.
//! Produces no output on success (designed for timing).
//!
//! In --check mode, outputs "OK formulas=N includes=0" to stdout on success.

use std::env;
use std::ffi::CString;
use std::process;

extern "C" {
    fn parse_tptp_file(path: *const std::os::raw::c_char) -> std::os::raw::c_int;
    fn parse_tptp_file_counted(
        path: *const std::os::raw::c_char,
        formula_count: *mut std::os::raw::c_int,
    ) -> std::os::raw::c_int;
}

fn main() {
    let args: Vec<String> = env::args().collect();

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

    let mut formula_count: std::os::raw::c_int = 0;
    let result = unsafe { parse_tptp_file_counted(c_path.as_ptr(), &mut formula_count) };

    match result {
        0 => {
            println!("OK formulas={} includes=0", formula_count);
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
