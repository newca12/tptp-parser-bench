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
    /// Parser only signals parse success/fail + counts; cross-validate against mrs-tptp
    /// for each count the parser reports.
    CrossValidate,
    /// Parser validity can be checked, but its internal unit counts are not
    /// comparable to the source-level formula counts from other parsers.
    ParseOnly,
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

/// Parse the standardized check output from subprocess wrappers:
/// "OK formulas=N includes=M" (or `includes=unknown` if unavailable).
pub fn parse_check_output(stdout: &str, elapsed_ms: f64) -> ParseResult {
    let line = stdout.trim();
    if line.starts_with("OK") {
        let mut formula_token = None;
        let mut include_token = None;
        for part in line.split_whitespace() {
            if let Some(val) = part.strip_prefix("formulas=") {
                formula_token = Some(val);
            }
            if let Some(val) = part.strip_prefix("includes=") {
                include_token = Some(val);
            }
        }
        match (
            formula_token.and_then(parse_count_token),
            include_token.and_then(parse_count_token),
        ) {
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
            formula_count: Some(formula_count),
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

fn parse_count_token(token: &str) -> Option<Option<usize>> {
    if token == "unknown" {
        Some(None)
    } else {
        token.parse().ok().map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_check_output;
    use crate::types::ParseResult;

    #[test]
    fn parses_known_include_count() {
        assert!(matches!(
            parse_check_output("OK formulas=12 includes=3", 1.0),
            ParseResult::Success {
                formula_count: Some(12),
                include_count: Some(3),
                ..
            }
        ));
    }

    #[test]
    fn accepts_unknown_include_count() {
        assert!(matches!(
            parse_check_output("OK formulas=12 includes=unknown", 1.0),
            ParseResult::Success {
                formula_count: Some(12),
                include_count: None,
                ..
            }
        ));
    }

    #[test]
    fn accepts_unknown_formula_and_include_counts() {
        assert!(matches!(
            parse_check_output("OK formulas=unknown includes=unknown", 1.0),
            ParseResult::Success {
                formula_count: None,
                include_count: None,
                ..
            }
        ));
    }
}
