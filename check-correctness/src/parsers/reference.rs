use std::path::Path;

/// Parse a file with mrs-tptp and return (formula_count, include_count).
/// Used as the reference oracle for CrossValidate parsers.
pub fn parse_file(file: &Path) -> Result<(usize, usize), String> {
    let content = std::fs::read_to_string(file).map_err(|e| format!("Read error: {}", e))?;

    let problem = mrs_tptp::parse_tptp(&content).map_err(|e| e.to_string())?;

    Ok((problem.formulas.len(), problem.includes.len()))
}
