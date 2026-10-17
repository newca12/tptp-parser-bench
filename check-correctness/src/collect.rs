use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collect all .p files in a directory, sorted by path
pub fn collect_tptp_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_recursive(&path, files)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "p") {
            files.push(path);
        }
    }
    Ok(())
}
