pub mod java_antlr;
pub mod mrs_tptp;
pub mod reference;
pub mod scala_tptp;
pub mod tptp_antlr;
pub mod tptp_nom;
pub mod vampire;

use crate::types::ParseResult;
use std::path::Path;

/// How this parser verifies correctness beyond just parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    /// Parser produces pretty-printed output; roundtrip = parse -> pretty -> reparse -> pretty -> compare
    Roundtrip,
    /// Parser only signals parse success/fail + formula count; cross-validate against mrs-tptp reference
    CrossValidate,
}

/// A parser backend that can check a single file for correctness.
pub trait ParserBackend: Send + Sync {
    /// Human-readable name of this parser
    fn name(&self) -> &str;

    /// How this parser's results should be verified
    fn verification_mode(&self) -> VerificationMode;

    /// Parse a single file and return the result.
    fn check_file(&self, file: &Path) -> ParseResult;
}

/// Parse the standardized check output from subprocess wrappers: "OK formulas=N includes=M"
pub fn parse_check_output(stdout: &str, elapsed_ms: f64) -> ParseResult {
    let line = stdout.trim();
    if line.starts_with("OK") {
        let mut formula_count = None;
        let mut include_count = None;
        for part in line.split_whitespace() {
            if let Some(val) = part.strip_prefix("formulas=") {
                formula_count = val.parse().ok();
            }
            if let Some(val) = part.strip_prefix("includes=") {
                include_count = val.parse().ok();
            }
        }
        match (formula_count, include_count) {
            (Some(formula_count), Some(include_count)) => ParseResult::Success {
                formula_count,
                include_count,
                elapsed_ms,
            },
            _ => ParseResult::Failed {
                elapsed_ms,
                error: format!("Malformed check output: {}", line),
            },
        }
    } else if line.starts_with("ROUNDTRIP_FAIL") {
        let mut formula_count = 0;
        let mut error = String::new();
        for part in line.splitn(2, "error=") {
            if part.starts_with("ROUNDTRIP_FAIL") {
                for token in part.split_whitespace() {
                    if let Some(val) = token.strip_prefix("formulas=") {
                        formula_count = val.parse().unwrap_or(0);
                    }
                }
            } else {
                error = part.to_string();
            }
        }
        ParseResult::RoundtripFail {
            formula_count,
            elapsed_ms,
            error,
        }
    } else {
        ParseResult::Failed {
            elapsed_ms,
            error: format!("Unexpected output: {}", line),
        }
    }
}
