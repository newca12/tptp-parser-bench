//! CLI wrapper for the `tptp` crate (nom-based parser by Michael Rawson).
//!
//! Usage: tptp-nom-wrapper <file.p>
//!
//! Reads the given TPTP file and parses it. Exits 0 on success, 1 on
//! parse error. Produces no output on success (designed for timing).

use std::env;
use std::fs;
use std::process;
use tptp::TPTPIterator;

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: tptp-nom-wrapper <file.p>");
            process::exit(2);
        }
    };

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error reading {}: {}", path, e);
            process::exit(2);
        }
    };

    let mut parser = TPTPIterator::<()>::new(&bytes);
    for input in &mut parser {
        if input.is_err() {
            eprintln!("syntax error in {}", path);
            process::exit(1);
        }
    }

    if !parser.remaining.is_empty() {
        eprintln!("trailing content in {}", path);
        process::exit(1);
    }
}
