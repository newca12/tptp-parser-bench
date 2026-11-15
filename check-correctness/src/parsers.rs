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
    let output = stdout.trim();
    let header = output
        .lines()
        .next()
        .unwrap_or_default()
        .trim_end_matches('\r');
    let mut parts = header.split_whitespace();

    match parts.next() {
        Some("OK") => {
            let mut formula_count = None;
            let mut include_count = None;
            let mut malformed = output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
                != 1;

            for part in parts {
                if let Some(value) = part.strip_prefix("formulas=") {
                    if formula_count.is_some() {
                        malformed = true;
                    }
                    formula_count = parse_count_token(value);
                    malformed |= formula_count.is_none();
                } else if let Some(value) = part.strip_prefix("includes=") {
                    if include_count.is_some() {
                        malformed = true;
                    }
                    include_count = parse_count_token(value);
                    malformed |= include_count.is_none();
                } else {
                    malformed = true;
                }
            }

            match (malformed, formula_count, include_count) {
                (false, Some(formula_count), Some(include_count)) => ParseResult::Success {
                    formula_count,
                    include_count,
                    elapsed_ms,
                },
                _ => ParseResult::Failed {
                    elapsed_ms,
                    error: format!("Malformed check output: {}", output),
                },
            }
        }
        Some("ROUNDTRIP_FAIL") => {
            let fields = header
                .strip_prefix("ROUNDTRIP_FAIL")
                .unwrap_or_default()
                .split("error=")
                .next()
                .unwrap_or_default();
            let mut formula_count = None;
            let mut malformed = false;

            for part in fields.split_whitespace() {
                if let Some(value) = part.strip_prefix("formulas=") {
                    if formula_count.is_some() {
                        malformed = true;
                    }
                    formula_count = value.parse().ok();
                    malformed |= formula_count.is_none();
                } else if let Some(value) = part.strip_prefix("includes=") {
                    malformed |= parse_count_token(value).is_none();
                } else {
                    malformed = true;
                }
            }

            let error = output
                .split_once("error=")
                .map(|(_, error)| error.trim())
                .filter(|error| !error.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    if malformed {
                        format!("Malformed ROUNDTRIP_FAIL output: {}", header)
                    } else {
                        "Roundtrip verification failed".to_string()
                    }
                });

            ParseResult::RoundtripFail {
                formula_count,
                elapsed_ms,
                error,
            }
        }
        _ => ParseResult::Failed {
            elapsed_ms,
            error: format!("Unexpected output: {}", output),
        },
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

    #[test]
    fn does_not_accept_status_tokens_that_only_start_with_ok() {
        assert!(matches!(
            parse_check_output("OKAY formulas=12 includes=3", 1.0),
            ParseResult::Failed { .. }
        ));
    }

    #[test]
    fn rejects_extra_fields_and_diagnostics_in_success_output() {
        assert!(matches!(
            parse_check_output("OK formulas=12 includes=3 extra=value", 1.0),
            ParseResult::Failed { .. }
        ));
        assert!(matches!(
            parse_check_output("OK formulas=12 includes=3\nunexpected diagnostic", 1.0),
            ParseResult::Failed { .. }
        ));
    }

    #[test]
    fn preserves_multiline_roundtrip_errors() {
        let result = parse_check_output(
            "ROUNDTRIP_FAIL formulas=12 includes=3 error=first line\nsecond line",
            1.0,
        );
        assert!(matches!(
            result,
            ParseResult::RoundtripFail {
                formula_count: Some(12),
                error,
                ..
            } if error == "first line\nsecond line"
        ));
    }
}
