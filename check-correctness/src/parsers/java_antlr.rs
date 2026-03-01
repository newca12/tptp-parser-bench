use crate::parsers::{parse_check_output, ParserBackend, VerificationMode};
use crate::types::ParseResult;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub struct JavaAntlrBackend {
    jar_path: PathBuf,
}

impl JavaAntlrBackend {
    pub fn new() -> Result<Self, String> {
        let path = PathBuf::from("parsers/java-antlr-wrapper/target/java-antlr-wrapper.jar");
        if !path.exists() {
            return Err(format!(
                "java-antlr-wrapper.jar not found at {}. Build with: cd parsers/java-antlr-wrapper && mvn package",
                path.display()
            ));
        }
        Ok(Self { jar_path: path })
    }
}

impl ParserBackend for JavaAntlrBackend {
    fn name(&self) -> &str {
        "tptp (ANTLR4, Java)"
    }

    fn verification_mode(&self) -> VerificationMode {
        VerificationMode::CrossValidate
    }

    fn check_file(&self, file: &Path) -> ParseResult {
        let start = Instant::now();

        let output = Command::new("java")
            .arg("-jar")
            .arg(&self.jar_path)
            .arg("--check")
            .arg(file)
            .output();

        let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;

        match output {
            Ok(out) => {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    parse_check_output(&stdout, elapsed_ms)
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    ParseResult::Failed {
                        elapsed_ms,
                        error: stderr.trim().to_string(),
                    }
                }
            }
            Err(e) => ParseResult::Failed {
                elapsed_ms,
                error: format!("Failed to spawn: {}", e),
            },
        }
    }
}
