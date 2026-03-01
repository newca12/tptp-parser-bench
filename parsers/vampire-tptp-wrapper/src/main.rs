//! CLI wrapper for Vampire's TPTP parser.
//!
//! Usage: vampire-tptp-wrapper <file.p>
//!
//! Reads the given TPTP file and parses it using Vampire's native
//! TPTP parser (Parse::TPTP). Exits 0 on success, 1 on parse error.
//! Produces no output on success (designed for timing).

use std::env;
use std::ffi::CString;
use std::process;

extern "C" {
    fn parse_tptp_file(path: *const std::os::raw::c_char) -> std::os::raw::c_int;
}

fn main() {
    let path = match env::args().nth(1) {
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
