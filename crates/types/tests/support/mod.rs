use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const LOCALES: &[&str] = &["zh-CN", "en", "zh-TW"];
pub const LOCALE_PARTS: &[&str] = &["common", "user", "rbac", "system", "captcha", "scheduler", "audit"];

pub fn parsed_catalogs() -> Vec<(&'static str, BTreeMap<String, String>)> {
    LOCALES.iter().map(|locale| (*locale, parse_parts(locale, LOCALE_PARTS))).collect()
}

pub fn parsed_responsibility_catalogs(part: &str) -> Vec<(&'static str, BTreeMap<String, String>)> {
    LOCALES.iter().map(|locale| (*locale, parse_parts(locale, &[part]))).collect()
}

fn parse_parts(locale: &str, parts: &[&str]) -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    for part in parts {
        let path = locale_path(part, locale);
        let source = fs::read_to_string(&path).unwrap_or_else(|error| panic!("failed to read locale file {}: {error}", path.display()));
        parse_source(&source, &path, &mut entries);
    }
    entries
}

fn locale_path(part: &str, locale: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("locales").join(part).join(format!("{locale}.yml"))
}

fn parse_source(source: &str, path: &Path, entries: &mut BTreeMap<String, String>) {
    for (index, line) in source.lines().enumerate() {
        if ignored_line(line) {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .unwrap_or_else(|| panic!("invalid locale entry at {}:{}", path.display(), index + 1));
        let key = key.trim().to_owned();
        let previous = entries.insert(key.clone(), value.trim().trim_matches('"').to_owned());
        assert!(previous.is_none(), "duplicate locale key {key} in {}", path.display());
    }
}

fn ignored_line(line: &str) -> bool {
    line.trim().is_empty() || line.trim_start().starts_with('#') || line.trim() == "_version: 1"
}

pub fn placeholders(value: &str) -> BTreeSet<String> {
    let mut remaining = value;
    let mut result = BTreeSet::new();
    while let Some(start) = remaining.find("%{") {
        let after_start = &remaining[start + 2..];
        let end = after_start.find('}').unwrap_or_else(|| panic!("unterminated placeholder in {value}"));
        result.insert(after_start[..end].to_owned());
        remaining = &after_start[end + 1..];
    }
    result
}
