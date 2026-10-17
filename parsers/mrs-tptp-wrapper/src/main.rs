//! CLI wrapper for the `mrs-tptp` crate (winnow-based parser).
//!
//! Usage: mrs-tptp-wrapper <file.p>
//!
//! Reads the given TPTP file and parses it. Exits 0 on success, 1 on
//! parse error. Produces no output on success (designed for timing).

use mrs_tptp::parse_tptp;
use std::env;
use std::fs;
use std::process;

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: mrs-tptp-wrapper <file.p>");
            process::exit(2);
        }
    };

    let content = match fs::read_to_string(&path) {
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
