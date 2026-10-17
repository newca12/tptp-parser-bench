use std::path::PathBuf;

/// Result of parsing a single file
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ParseResult {
    /// Parse + verification succeeded
    Success {
        formula_count: usize,
        include_count: usize,
        elapsed_ms: f64,
    },
    /// Parse succeeded but roundtrip (or cross-validation) failed
    RoundtripFail {
        formula_count: usize,
        elapsed_ms: f64,
        error: String,
    },
    /// Parse itself failed
    Failed { elapsed_ms: f64, error: String },
    /// Could not read file
    ReadError { error: String },
    /// File exceeded timeout
    Timeout { timeout_ms: u64 },
}

/// Complete result for one file
#[derive(Debug)]
pub struct FileResult {
    pub path: PathBuf,
    pub relative_path: String,
    pub result: ParseResult,
}

/// Aggregated summary statistics
#[derive(Debug, Default)]
pub struct CheckSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub timeouts: usize,
    pub roundtrip_failures: usize,
    pub total_formulas: usize,
    pub total_includes: usize,
    pub total_parse_time_ms: f64,
    pub wall_time_secs: f64,
    pub failed_files: Vec<(PathBuf, String)>,
    pub roundtrip_failed_files: Vec<(PathBuf, String, String)>,
    pub timeout_files: Vec<(PathBuf, u64)>,
}
