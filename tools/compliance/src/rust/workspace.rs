use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    report::{Violation, ViolationDetails},
    rust::analysis::{analyze_source, is_production_rust_file, relative_path},
};

const BACKEND_SOURCE_PATH: &str = "apps/backend/src";
const CRATES_PATH: &str = "crates";
const TOOLS_SOURCE_PATH: &str = "tools/compliance/src";

pub fn scan_workspace(root: &Path) -> Vec<Violation> {
    let mut violations = Vec::new();
    for source_root in source_roots(root) {
        scan_directory(root, &source_root, &mut violations);
    }
    violations
}

fn source_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = vec![root.join(BACKEND_SOURCE_PATH), root.join(TOOLS_SOURCE_PATH)];
    let crates = root.join(CRATES_PATH);
    let Ok(entries) = fs::read_dir(crates) else {
        return roots;
    };
    roots.extend(entries.flatten().map(|entry| entry.path().join("src")).filter(|path| path.is_dir()));
    roots.into_iter().filter(|path| path.is_dir()).collect()
}

fn scan_directory(root: &Path, directory: &Path, violations: &mut Vec<Violation>) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            violations.push(io_violation(root, directory, error));
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(root, &path, violations);
        } else if is_production_rust_file(&path) {
            scan_file(root, &path, violations);
        }
    }
}

fn scan_file(root: &Path, path: &Path, violations: &mut Vec<Violation>) {
    let relative = relative_path(root, path);
    match fs::read_to_string(path) {
        Ok(source) => violations.extend(analyze_source(relative, &source)),
        Err(error) => violations.push(io_violation(root, path, error)),
    }
}

fn io_violation(root: &Path, path: &Path, error: std::io::Error) -> Violation {
    Violation::new(
        relative_path(root, path),
        ViolationDetails {
            line: 1,
            rule: "rust-io",
            message: error.to_string(),
        },
    )
}
