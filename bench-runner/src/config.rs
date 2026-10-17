use serde::Deserialize;
use std::io;

/// Top-level configuration loaded from `parsers.toml`.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub settings: Settings,
    pub parsers: Vec<ParserConfig>,
}

/// Global benchmark settings.
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_iterations")]
    pub iterations: usize,
    #[serde(default = "default_warmup")]
    pub warmup_iterations: usize,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

/// A single parser to benchmark.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ParserConfig {
    pub name: String,
    pub display_name: String,
    pub language: String,
    pub technique: String,
    #[serde(default)]
    pub repository: String,
    pub run_cmd: Vec<String>,
    #[serde(default = "default_input_mode")]
    pub input_mode: InputMode,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// How the parser receives its input.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    /// File path is passed as the last command-line argument.
    FileArg,
    /// File content is piped to stdin.
    Stdin,
}

fn default_iterations() -> usize {
    50
}
fn default_warmup() -> usize {
    5
}
fn default_timeout() -> u64 {
    30
}
fn default_input_mode() -> InputMode {
    InputMode::FileArg
}
fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        if config.settings.iterations == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "settings.iterations must be greater than zero",
            )
            .into());
        }
        if config.settings.timeout_secs == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "settings.timeout_secs must be greater than zero",
            )
            .into());
        }
        for parser in &config.parsers {
            if parser.run_cmd.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("parser '{}' has an empty run_cmd", parser.name),
                )
                .into());
            }
        }
        Ok(config)
    }

    /// Return only enabled parsers.
    pub fn enabled_parsers(&self) -> Vec<&ParserConfig> {
        self.parsers.iter().filter(|p| p.enabled).collect()
    }
}
