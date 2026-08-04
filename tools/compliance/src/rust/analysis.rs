use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::{
    Attribute, File, Meta,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::{
    report::{Violation, ViolationDetails},
    rust::visitor::RustVisitor,
};

const BACKEND_BUSINESS_LAYERS: [&str; 4] = ["api", "application", "domain", "infra"];

pub fn analyze_source(path: PathBuf, source: &str) -> Vec<Violation> {
    let parsed = match syn::parse_file(source) {
        Ok(parsed) => parsed,
        Err(error) => return vec![parse_violation(path, error)],
    };
    let mut violations = backend_ownership_violations(&path);
    violations.extend(file_line_violations(&path, source, &parsed));
    RustVisitor::new(path, source, &mut violations).visit_file(&parsed);
    violations
}

pub(crate) fn is_production_rust_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "rs") && !is_test_path(path)
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

pub(crate) fn span_line(span: Span) -> usize {
    span.start().line.max(1)
}

pub(crate) fn is_test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(is_test_attribute)
}

fn parse_violation(path: PathBuf, error: syn::Error) -> Violation {
    Violation::new(
        path,
        ViolationDetails {
            line: span_line(error.span()),
            rule: "rust-parse",
            message: error.to_string(),
        },
    )
}

fn is_test_path(path: &Path) -> bool {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    let is_test_file = name == "tests.rs" || name.ends_with("_tests.rs") || name.starts_with("test_") || name == "test_support.rs";
    is_test_file || path.components().any(|component| is_test_component(component.as_os_str().to_str()))
}

fn is_test_component(component: Option<&str>) -> bool {
    component.is_some_and(|name| name == "tests" || name == "test_support" || name.ends_with("_tests"))
}

fn is_test_attribute(attribute: &Attribute) -> bool {
    attribute.path().segments.last().is_some_and(|segment| segment.ident == "test")
        || matches!(&attribute.meta, Meta::List(list) if attribute.path().is_ident("cfg") && list.tokens.to_string() == "test")
}

fn backend_ownership_violations(path: &Path) -> Vec<Violation> {
    let mut components = path.components();
    let is_backend_source = matches!(components.next(), Some(component) if component.as_os_str() == "apps")
        && matches!(components.next(), Some(component) if component.as_os_str() == "backend")
        && matches!(components.next(), Some(component) if component.as_os_str() == "src");
    if !is_backend_source {
        return Vec::new();
    }
    let Some(layer) = components.next().and_then(|component| component.as_os_str().to_str()) else {
        return Vec::new();
    };
    if !BACKEND_BUSINESS_LAYERS.contains(&layer) {
        return Vec::new();
    }
    vec![Violation::new(
        path.to_path_buf(),
        ViolationDetails {
            line: 1,
            rule: "backend-composition-ownership",
            message: format!("apps/backend composition root must not own {layer} business code"),
        },
    )]
}

fn file_line_violations(path: &Path, source: &str, parsed: &File) -> Vec<Violation> {
    let production_lines = production_line_count(source, &test_ranges(parsed));
    if production_lines <= crate::rust::MAX_FILE_LINES {
        return Vec::new();
    }
    vec![Violation::new(
        path.to_path_buf(),
        ViolationDetails {
            line: 1,
            rule: "rust-file-lines",
            message: format!("production file has {production_lines} lines (max {})", crate::rust::MAX_FILE_LINES),
        },
    )]
}

fn test_ranges(file: &File) -> Vec<LineRange> {
    let mut collector = TestRangeCollector::default();
    collector.visit_file(file);
    collector.ranges
}

#[derive(Default)]
struct TestRangeCollector {
    ranges: Vec<LineRange>,
}

impl<'ast> Visit<'ast> for TestRangeCollector {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if self.collect_if_test_only(&item.attrs, item.span()) {
            return;
        }
        visit::visit_item_fn(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if self.collect_if_test_only(&item.attrs, item.span()) {
            return;
        }
        visit::visit_item_mod(self, item);
    }
}

impl TestRangeCollector {
    fn collect_if_test_only(&mut self, attributes: &[Attribute], span: Span) -> bool {
        if !is_test_only(attributes) {
            return false;
        }
        self.ranges.push(LineRange::from_span(span));
        true
    }
}

fn production_line_count(source: &str, test_ranges: &[LineRange]) -> usize {
    source
        .lines()
        .enumerate()
        .filter(|(index, _)| !line_is_test_only(index + 1, test_ranges))
        .count()
}

fn line_is_test_only(line: usize, ranges: &[LineRange]) -> bool {
    ranges.iter().any(|range| range.contains(line))
}

#[derive(Clone, Copy)]
struct LineRange {
    start: usize,
    end: usize,
}

impl LineRange {
    fn from_span(span: Span) -> Self {
        Self {
            start: span_line(span),
            end: span.end().line.max(1),
        }
    }

    fn contains(self, line: usize) -> bool {
        (self.start..=self.end).contains(&line)
    }
}
