use crate::parsers::{ParserBackend, VerificationMode, parse_check_output};
use crate::types::ParseResult;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub struct ScalaTptpBackend {
    jar_path: PathBuf,
}

impl ScalaTptpBackend {
    pub fn new() -> Result<Self, String> {
        let path =
            PathBuf::from("parsers/scala-tptp-wrapper/target/scala-3.9.0/scala-tptp-wrapper.jar");
        if !path.exists() {
            return Err(format!(
                "scala-tptp-wrapper.jar not found at {}. Build with: cd parsers/scala-tptp-wrapper && sbt assembly",
                path.display()
            ));
        }
        Ok(Self { jar_path: path })
    }
}

impl ParserBackend for ScalaTptpBackend {
    fn name(&self) -> &str {
        "scala-tptp-parser (combinators)"
    }

    fn verification_mode(&self) -> VerificationMode {
        // Scala wrapper does roundtrip internally
        VerificationMode::Roundtrip
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
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // Check if stdout has ROUNDTRIP_FAIL
                    let trimmed_stdout = stdout.trim();
                    if trimmed_stdout.starts_with("ROUNDTRIP_FAIL") {
                        parse_check_output(trimmed_stdout, elapsed_ms)
                    } else {
                        ParseResult::Failed {
                            elapsed_ms,
                            error: stderr.trim().to_string(),
                        }
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
