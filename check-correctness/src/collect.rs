use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collect all .p files in a directory, sorted by path
pub fn collect_tptp_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_recursive(&path, files);
                } else if path.extension().map_or(false, |ext| ext == "p") {
                    files.push(path);
                }
            }
        }
    }
}
