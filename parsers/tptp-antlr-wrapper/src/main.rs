//! CLI wrapper for the ANTLR4-based TPTP parser (from playground-rs).
//!
//! Usage: tptp-antlr-wrapper <file.p>
//!
//! Reads the given TPTP file and parses the entire file using the ANTLR4
//! generated parser. Exits 0 on success, 1 on parse error.
//! Produces no output on success (designed for timing).

use std::env;
use std::fs;
use std::process;

use tptp_antlr::tptplexer::TPTPLexer;
use tptp_antlr::tptpparser::TPTPParser;

use antlr4rust::InputStream;
use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::token_factory::CommonTokenFactory;

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: tptp-antlr-wrapper <file.p>");
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

    let tf = CommonTokenFactory;
    let lexer = TPTPLexer::new_with_token_factory(InputStream::new(&*content), &tf);
    let token_source = CommonTokenStream::new(lexer);
    let mut parser = TPTPParser::new(token_source);
    let success = parser.tptp_file().is_ok();
    // Drop parser results before locals they borrow from.
    drop(parser);

    if !success {
        eprintln!("parse error in {}", path);
        process::exit(1);
    }
}
