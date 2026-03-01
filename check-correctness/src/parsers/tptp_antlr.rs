use crate::parsers::{ParserBackend, VerificationMode};
use crate::types::ParseResult;
use std::path::Path;
use std::time::Instant;

use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::token_factory::CommonTokenFactory;
use antlr4rust::InputStream;
use tptp_antlr::tptplexer::TPTPLexer;
use tptp_antlr::tptpparser::{TPTPParser, Tptp_fileContextAttrs, Tptp_inputContextAttrs};

pub struct TptpAntlrBackend;

impl ParserBackend for TptpAntlrBackend {
    fn name(&self) -> &str {
        "tptp (ANTLR4)"
    }

    fn verification_mode(&self) -> VerificationMode {
        VerificationMode::CrossValidate
    }

    fn check_file(&self, file: &Path) -> ParseResult {
        let content = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                return ParseResult::ReadError {
                    error: format!("Read error: {}", e),
                }
            }
        };

        let start = Instant::now();

        // Extract counts in a block so all ANTLR borrows are dropped before we return
        let result: Result<(usize, usize, f64), f64> = {
            let tf = CommonTokenFactory;
            let lexer = TPTPLexer::new_with_token_factory(InputStream::new(&*content), &tf);
            let token_source = CommonTokenStream::new(lexer);
            let mut parser = TPTPParser::new(token_source);

            let parse_result = parser.tptp_file();
            let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;

            match parse_result {
                Ok(ctx) => {
                    let mut formula_count = 0usize;
                    let mut include_count = 0usize;

                    let inputs = ctx.tptp_input_all();
                    for input_ctx in &inputs {
                        if input_ctx.annotated_formula().is_some() {
                            formula_count += 1;
                        } else if input_ctx.include().is_some() {
                            include_count += 1;
                        }
                    }

                    // Explicitly drop parse tree and parser before block ends
                    drop(inputs);
                    drop(ctx);
                    drop(parser);

                    Ok((formula_count, include_count, elapsed_ms))
                }
                Err(_) => {
                    drop(parser);
                    Err(elapsed_ms)
                }
            }
        };

        match result {
            Ok((formula_count, include_count, elapsed_ms)) => ParseResult::Success {
                formula_count,
                include_count,
                elapsed_ms,
            },
            Err(elapsed_ms) => ParseResult::Failed {
                elapsed_ms,
                error: "Parse error".to_string(),
            },
        }
    }
}
