use crate::parsers::{parse_check_output, ParserBackend, VerificationMode};
use crate::types::ParseResult;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub struct VampireBackend {
    wrapper_path: PathBuf,
}

impl VampireBackend {
    pub fn new() -> Result<Self, String> {
        let path = PathBuf::from("./target/release/vampire-tptp-wrapper");
        if !path.exists() {
            return Err(format!(
                "vampire-tptp-wrapper not found at {}. Build with: cargo build --release -p vampire-tptp-wrapper",
                path.display()
            ));
        }
        Ok(Self { wrapper_path: path })
    }
}

impl ParserBackend for VampireBackend {
    fn name(&self) -> &str {
        "Vampire TPTP (hand-written)"
    }

    fn verification_mode(&self) -> VerificationMode {
        VerificationMode::CrossValidate
    }

    fn check_file(&self, file: &Path) -> ParseResult {
        let start = Instant::now();

        let output = Command::new(&self.wrapper_path)
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
