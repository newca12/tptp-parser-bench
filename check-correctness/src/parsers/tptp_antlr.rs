use crate::parsers::{ParserBackend, VerificationMode};
use crate::types::ParseResult;
use antlr4rust::error_listener::ErrorListener;
use antlr4rust::errors::ANTLRError;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token_factory::TokenFactory;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

use antlr4rust::InputStream;
use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::token_factory::CommonTokenFactory;
use tptp_antlr::tptplexer::TPTPLexer;
use tptp_antlr::tptpparser::{TPTPParser, Tptp_fileContextAttrs, Tptp_inputContextAttrs};

#[derive(Clone, Default)]
struct SyntaxErrors(Rc<RefCell<Option<String>>>);

impl SyntaxErrors {
    fn first(&self) -> Option<String> {
        self.0.borrow().clone()
    }
}

impl<'input, T: Recognizer<'input>> ErrorListener<'input, T> for SyntaxErrors {
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<&<T::TF as TokenFactory<'input>>::Inner>,
        line: isize,
        column: isize,
        message: &str,
        _error: Option<&ANTLRError>,
    ) {
        let mut first = self.0.borrow_mut();
        if first.is_none() {
            *first = Some(format!("line {line}:{column} {message}"));
        }
    }
}

pub struct TptpAntlrBackend;

impl ParserBackend for TptpAntlrBackend {
    fn name(&self) -> &str {
        "tptp (ANTLR4, Rust)"
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
                };
            }
        };

        let start = Instant::now();

        // Extract counts in a block so all ANTLR borrows are dropped before we return
        let result: Result<(usize, usize, f64), (f64, String)> = {
            let tf = CommonTokenFactory;
            let errors = SyntaxErrors::default();
            let mut lexer = TPTPLexer::new_with_token_factory(InputStream::new(&*content), &tf);
            lexer.remove_error_listeners();
            lexer.add_error_listener(Box::new(errors.clone()));
            let token_source = CommonTokenStream::new(lexer);
            let mut parser = TPTPParser::new(token_source);
            parser.remove_error_listeners();
            parser.add_error_listener(Box::new(errors.clone()));

            let parse_result = parser.tptp_file();
            let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;
            let syntax_error = errors.first();

            match (parse_result, syntax_error) {
                (Ok(ctx), None) => {
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
                (Ok(ctx), Some(error)) => {
                    drop(ctx);
                    drop(parser);
                    Err((elapsed_ms, error))
                }
                (Err(error), syntax_error) => {
                    drop(parser);
                    Err((
                        elapsed_ms,
                        syntax_error.unwrap_or_else(|| format!("Parse error: {error:?}")),
                    ))
                }
            }
        };

        match result {
            Ok((formula_count, include_count, elapsed_ms)) => ParseResult::Success {
                formula_count: Some(formula_count),
                include_count: Some(include_count),
                elapsed_ms,
            },
            Err((elapsed_ms, error)) => ParseResult::Failed { elapsed_ms, error },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TptpAntlrBackend;
    use crate::parsers::ParserBackend;
    use crate::types::ParseResult;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn check_source(source: &str) -> ParseResult {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "check-correctness-antlr-{}-{unique}.p",
            std::process::id()
        ));
        std::fs::write(&path, source).expect("write test input");
        let result = TptpAntlrBackend.check_file(&path);
        let _ = std::fs::remove_file(path);
        result
    }

    #[test]
    fn rejects_syntax_errors_even_when_antlr_recovers_a_parse_tree() {
        assert!(matches!(
            check_source("fof(bad,axiom,p &)."),
            ParseResult::Failed { .. }
        ));
    }

    #[test]
    fn accepts_a_syntactically_valid_formula() {
        assert!(matches!(
            check_source("fof(good,axiom,p)."),
            ParseResult::Success { .. }
        ));
    }
}
