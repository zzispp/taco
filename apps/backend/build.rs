use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

const DEFAULT_LOCALE_ENTRY_DOCUMENT_NAME: &str = "index.html";
const NOT_FOUND_DOCUMENT_NAME: &str = "404.html";
const LOCALE_NOT_FOUND_DOCUMENT_PATH: &str = "error/404/index.html";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LocaleContract {
    default_locale: String,
    locales: Vec<Locale>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Locale {
    code: String,
    document_language: String,
    backend_language: String,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies CARGO_MANIFEST_DIR"));
    let contract_path = manifest_dir.join("../../locale-contract.json");
    println!("cargo:rerun-if-changed={}", contract_path.display());
    let contract = read_locale_contract(&contract_path);
    write_embedded_frontend_contract(&contract);

    if env::var_os("CARGO_FEATURE_EMBEDDED_FRONTEND").is_none() {
        return;
    }

    let output_dir = manifest_dir.join("../frontend/out");
    println!("cargo:rerun-if-changed={}", output_dir.display());
    for required_file in required_documents(&contract) {
        let path = output_dir.join(required_file);
        println!("cargo:rerun-if-changed={}", path.display());
        assert!(
            path.is_file(),
            "embedded frontend requires {}; run `pnpm --filter frontend build:embedded` before building with --features embedded-frontend",
            path.display()
        );
    }
}

fn read_locale_contract(path: &Path) -> LocaleContract {
    let contents = fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read locale contract {}: {error}", path.display()));
    let contract = serde_json::from_str::<LocaleContract>(&contents).unwrap_or_else(|error| panic!("invalid locale contract {}: {error}", path.display()));
    validate_locale_contract(&contract);
    contract
}

fn validate_locale_contract(contract: &LocaleContract) {
    assert!(!contract.locales.is_empty(), "locale contract must include at least one locale");
    assert!(
        contract.locales.iter().any(|locale| locale.code == contract.default_locale),
        "locale contract defaultLocale must be listed in locales"
    );
    for locale in &contract.locales {
        assert!(!locale.code.is_empty(), "locale contract locale code cannot be empty");
        assert!(!locale.document_language.is_empty(), "locale contract documentLanguage cannot be empty");
        assert!(!locale.backend_language.is_empty(), "locale contract backendLanguage cannot be empty");
    }
}

fn write_embedded_frontend_contract(contract: &LocaleContract) {
    let output_path = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR")).join("embedded_frontend_contract.rs");
    let supported_locales = contract
        .locales
        .iter()
        .map(|locale| format!("{:?}", locale.code))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "pub const DEFAULT_LOCALE_ROOT_PATH: &str = \"/{}/\";\npub const SUPPORTED_LOCALES: &[&str] = &[{}];\n",
        contract.default_locale, supported_locales
    );
    fs::write(&output_path, source).unwrap_or_else(|error| panic!("failed to write generated locale contract {}: {error}", output_path.display()));
}

fn required_documents(contract: &LocaleContract) -> Vec<String> {
    let mut documents = vec![
        format!("{}/{}", contract.default_locale, DEFAULT_LOCALE_ENTRY_DOCUMENT_NAME),
        NOT_FOUND_DOCUMENT_NAME.into(),
    ];
    documents.extend(
        contract
            .locales
            .iter()
            .map(|locale| format!("{}/{LOCALE_NOT_FOUND_DOCUMENT_PATH}", locale.code)),
    );
    documents
}
