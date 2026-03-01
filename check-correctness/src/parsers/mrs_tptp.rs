use crate::parsers::{ParserBackend, VerificationMode};
use crate::types::ParseResult;
use mrs_tptp::parse_tptp;
use std::path::Path;
use std::time::Instant;

pub struct MrsTptpBackend;

impl ParserBackend for MrsTptpBackend {
    fn name(&self) -> &str {
        "mrs-tptp (winnow)"
    }

    fn verification_mode(&self) -> VerificationMode {
        VerificationMode::Roundtrip
    }

    fn check_file(&self, file: &Path) -> ParseResult {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                return ParseResult::ReadError {
                    error: format!("Read error: {}", e),
                }
            }
        };

        let start = Instant::now();
        let result = parse_tptp(&content);
        let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;

        match result {
            Ok(problem) => {
                let formula_count = problem.formulas.len();
                let include_count = problem.includes.len();

                // Roundtrip: for each formula, pretty-print -> reparse -> compare
                for formula in &problem.formulas {
                    let name = formula.name().to_string();
                    let pretty = formula.to_string();

                    match parse_tptp(&pretty) {
                        Ok(reparsed_problem) => {
                            if reparsed_problem.formulas.len() == 1 {
                                let reparsed_pretty =
                                    reparsed_problem.formulas[0].to_string();
                                if pretty != reparsed_pretty {
                                    return ParseResult::RoundtripFail {
                                        formula_count,
                                        elapsed_ms,
                                        error: format!(
                                            "Round-trip mismatch for '{}':\n  Original:  {}\n  Reparsed:  {}",
                                            name, pretty, reparsed_pretty
                                        ),
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            return ParseResult::RoundtripFail {
                                formula_count,
                                elapsed_ms,
                                error: format!(
                                    "Failed to reparse '{}': {}\n  Pretty: {}",
                                    name, e, pretty
                                ),
                            };
                        }
                    }
                }

                ParseResult::Success {
                    formula_count,
                    include_count,
                    elapsed_ms,
                }
            }
            Err(e) => ParseResult::Failed {
                elapsed_ms,
                error: e.to_string(),
            },
        }
    }
}
