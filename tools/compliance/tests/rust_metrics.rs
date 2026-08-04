use std::path::PathBuf;

use compliance::rust::{MAX_FILE_LINES, analyze_source};

#[test]
fn accepts_the_valid_rust_fixture() {
    let violations = analyze_source(PathBuf::from("valid.rs"), include_str!("fixtures/valid.rs"));

    assert!(violations.is_empty());
}

#[test]
fn reports_each_rust_fixture_violation_with_its_source_location() {
    let violations = analyze_source(PathBuf::from("invalid.rs"), include_str!("fixtures/invalid.rs"));
    let rules = violations.iter().map(|violation| violation.rule).collect::<Vec<_>>();

    assert!(rules.contains(&"rust-positional-parameters"));
    assert!(rules.contains(&"rust-nesting-depth"));
    assert!(rules.contains(&"rust-panic-prone-call"));
    assert!(violations.iter().all(|violation| violation.line > 0));
}

#[test]
fn reports_backend_business_code_placed_in_the_composition_root() {
    let violations = analyze_source(PathBuf::from("apps/backend/src/domain/user.rs"), "pub fn user_rule() {}");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "backend-composition-ownership");
    assert_eq!(violations[0].line, 1);
}

#[test]
fn reports_rust_function_file_and_complexity_limits_from_fixtures() {
    let source = include_str!("fixtures/limits.rs");
    let violations = analyze_source(PathBuf::from("limits.rs"), source);
    let rule_locations = violations.iter().map(|violation| (violation.rule, violation.line)).collect::<Vec<_>>();

    assert_eq!(rule_locations, vec![("rust-function-lines", 1), ("rust-cyclomatic-complexity", 54),]);

    let oversized_source = format!("{source}\n{}", "\n".repeat(MAX_FILE_LINES));
    let oversized_violations = analyze_source(PathBuf::from("oversized.rs"), &oversized_source);
    assert_eq!(
        oversized_violations
            .iter()
            .filter(|violation| violation.rule == "rust-file-lines")
            .map(|violation| violation.line)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn reports_closure_metrics_without_charging_them_to_the_enclosing_function() {
    let violations = analyze_source(PathBuf::from("closures.rs"), include_str!("fixtures/closures.rs"));
    let rule_locations = violations.iter().map(|violation| (violation.rule, violation.line)).collect::<Vec<_>>();

    assert_eq!(rule_locations, vec![("rust-positional-parameters", 2), ("rust-cyclomatic-complexity", 5),]);
}

#[test]
fn excludes_nested_test_modules_from_production_file_size_measurement() {
    let source = format!(
        "pub fn production() {{}}\n#[cfg(test)]\nmod tests {{\n{}\n}}",
        "// test-only\n".repeat(MAX_FILE_LINES + 1)
    );
    let violations = analyze_source(PathBuf::from("nested_tests.rs"), &source);

    assert_eq!(violations.iter().filter(|violation| violation.rule == "rust-file-lines").count(), 0);
}

#[test]
fn selected_modules_meet_the_file_and_function_size_limits() {
    assert_module_size_compliance("apps/backend/src/composition.rs", include_str!("../../../apps/backend/src/composition.rs"));
    assert_module_size_compliance(
        "crates/tracing/src/http_capture.rs",
        include_str!("../../../crates/tracing/src/http_capture.rs"),
    );
    assert_module_size_compliance(
        "crates/system/src/application/service/use_cases.rs",
        include_str!("../../../crates/system/src/application/service/use_cases.rs"),
    );
}

fn assert_module_size_compliance(path: &str, source: &str) {
    let violations = analyze_source(PathBuf::from(path), source);
    let size_violations = violations
        .iter()
        .filter(|violation| matches!(violation.rule, "rust-file-lines" | "rust-function-lines"))
        .collect::<Vec<_>>();

    assert!(size_violations.is_empty(), "{size_violations:?}");
}
