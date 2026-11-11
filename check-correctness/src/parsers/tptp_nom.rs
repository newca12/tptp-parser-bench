use crate::parsers::{ParserBackend, VerificationMode};
use crate::types::ParseResult;
use std::path::Path;
use std::time::Instant;
use tptp::TPTPIterator;
use tptp::top::{AnnotatedFormula, TPTPInput};

pub struct TptpNomBackend;

impl ParserBackend for TptpNomBackend {
    fn name(&self) -> &str {
        "tptp (nom, zero-copy)"
    }

    fn verification_mode(&self) -> VerificationMode {
        VerificationMode::Roundtrip
    }

    fn check_file(&self, file: &Path) -> ParseResult {
        let bytes = match std::fs::read(file) {
            Ok(b) => b,
            Err(e) => {
                return ParseResult::ReadError {
                    error: format!("Read error: {}", e),
                };
            }
        };

        let start = Instant::now();
        let mut parser = TPTPIterator::<()>::new(&bytes);
        let mut formula_count = 0usize;
        let mut include_count = 0usize;
        let mut items_text: Vec<(String, String)> = Vec::new(); // (name_or_label, display_text)

        for input in &mut parser {
            match input {
                Ok(item) => {
                    let text = format!("{}", item);
                    match &item {
                        TPTPInput::Annotated(af) => {
                            formula_count += 1;
                            let name = get_formula_name(af);
                            items_text.push((name, text));
                        }
                        TPTPInput::Include(_) => {
                            include_count += 1;
                        }
                    }
                }
                Err(_) => {
                    let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;
                    return ParseResult::Failed {
                        elapsed_ms,
                        error: "Syntax error".to_string(),
                    };
                }
            }
        }

        if !parser.remaining.is_empty() {
            let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;
            return ParseResult::Failed {
                elapsed_ms,
                error: "Trailing content".to_string(),
            };
        }

        let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;

        // Roundtrip: reparse each formula's Display output
        for (name, text) in &items_text {
            let bytes2 = text.as_bytes();
            let mut parser2 = TPTPIterator::<()>::new(bytes2);
            let reparsed: Vec<_> = parser2.by_ref().collect();

            if !parser2.remaining.is_empty() || reparsed.len() != 1 {
                return ParseResult::RoundtripFail {
                    formula_count: Some(formula_count),
                    elapsed_ms,
                    error: format!("Failed to reparse '{}':\n  Original: {}", name, text),
                };
            }

            match &reparsed[0] {
                Ok(reparsed_item) => {
                    let reparsed_text = format!("{}", reparsed_item);
                    if *text != reparsed_text {
                        return ParseResult::RoundtripFail {
                            formula_count: Some(formula_count),
                            elapsed_ms,
                            error: format!(
                                "Round-trip mismatch for '{}':\n  Original:  {}\n  Reparsed:  {}",
                                name, text, reparsed_text
                            ),
                        };
                    }
                }
                Err(_) => {
                    return ParseResult::RoundtripFail {
                        formula_count: Some(formula_count),
                        elapsed_ms,
                        error: format!("Failed to reparse '{}':\n  Original: {}", name, text),
                    };
                }
            }
        }

        ParseResult::Success {
            formula_count: Some(formula_count),
            include_count: Some(include_count),
            elapsed_ms,
        }
    }
}

fn get_formula_name(af: &AnnotatedFormula) -> String {
    match af {
        AnnotatedFormula::Tfx(t) => format!("{}", t.0.name),
        AnnotatedFormula::Fof(f) => format!("{}", f.0.name),
        AnnotatedFormula::Cnf(c) => format!("{}", c.0.name),
    }
}
