use std::{
    env, fs,
    path::{Path, PathBuf},
};

const LOCALE_DIR: &str = "locales";
const LOCALE_EXTENSIONS: &[&str] = &["yml", "yaml", "json", "toml"];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is required"));
    let locale_dir = manifest_dir.join(LOCALE_DIR);
    assert!(locale_dir.is_dir(), "types locale directory is missing: {}", locale_dir.display());

    emit_rerun_path(&locale_dir);
    emit_locale_files(&locale_dir);
}

fn emit_locale_files(dir: &Path) {
    for path in sorted_entries(dir) {
        if path.is_dir() {
            emit_rerun_path(&path);
            emit_locale_files(&path);
            continue;
        }
        if is_locale_file(&path) {
            emit_rerun_path(&path);
        }
    }
}

fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read locale directory {}: {error}", dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read locale entry in {}: {error}", dir.display()))
                .path()
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn is_locale_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| LOCALE_EXTENSIONS.contains(&extension))
}

fn emit_rerun_path(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
}
