use proc_macro2::Span;
use syn::{
    Expr, ExprClosure, FnArg, ImplItemFn, ItemFn, ItemMod, TraitItemFn,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::{
    report::{Violation, ViolationDetails},
    rust::{
        MAX_CYCLOMATIC_COMPLEXITY, MAX_FUNCTION_LINES, MAX_NESTING_DEPTH, MAX_POSITIONAL_PARAMETERS,
        analysis::{is_test_only, span_line},
        metrics::{FunctionMetrics, closure_metrics, function_metrics},
    },
};

const PANIC_METHODS: [&str; 2] = ["unwrap", "expect"];

pub(crate) struct RustVisitor<'source, 'violations> {
    path: std::path::PathBuf,
    source: &'source str,
    violations: &'violations mut Vec<Violation>,
}

struct FunctionDetails<'ast> {
    name: &'ast syn::Ident,
    input_count: usize,
    body: &'ast syn::Block,
    span: Span,
}

impl<'source, 'violations> RustVisitor<'source, 'violations> {
    pub(crate) fn new(path: std::path::PathBuf, source: &'source str, violations: &'violations mut Vec<Violation>) -> Self {
        Self { path, source, violations }
    }

    fn assess_function(&mut self, function: FunctionDetails<'_>) {
        let name = function.name.to_string();
        self.check_function_lines(&name, function.span);
        self.check_parameter_count(&name, function.input_count, function.span);
        self.check_function_metrics(&name, function_metrics(function.body), function.span);
    }

    fn check_function_lines(&mut self, name: &str, span: Span) {
        let lines = nonblank_line_count(self.source, span);
        if lines > MAX_FUNCTION_LINES {
            self.push(
                span,
                "rust-function-lines",
                format!("function {name} has {lines} nonblank lines (max {MAX_FUNCTION_LINES})"),
            );
        }
    }

    fn check_parameter_count(&mut self, name: &str, count: usize, span: Span) {
        if count > MAX_POSITIONAL_PARAMETERS {
            self.push(
                span,
                "rust-positional-parameters",
                format!("function {name} has {count} positional parameters (max {MAX_POSITIONAL_PARAMETERS})"),
            );
        }
    }

    fn check_function_metrics(&mut self, name: &str, metrics: FunctionMetrics, span: Span) {
        if metrics.max_nesting > MAX_NESTING_DEPTH {
            self.push(
                span,
                "rust-nesting-depth",
                format!("function {name} has nesting depth {} (max {MAX_NESTING_DEPTH})", metrics.max_nesting),
            );
        }
        if metrics.cyclomatic_complexity > MAX_CYCLOMATIC_COMPLEXITY {
            self.push(
                span,
                "rust-cyclomatic-complexity",
                format!(
                    "function {name} has cyclomatic complexity {} (max {MAX_CYCLOMATIC_COMPLEXITY})",
                    metrics.cyclomatic_complexity
                ),
            );
        }
    }

    fn push(&mut self, span: Span, rule: &'static str, message: impl Into<String>) {
        self.violations.push(Violation::new(
            self.path.clone(),
            ViolationDetails {
                line: span_line(span),
                rule,
                message: message.into(),
            },
        ));
    }

    fn inspect_panic_method(&mut self, method: &syn::Ident, span: Span) {
        if PANIC_METHODS.contains(&method.to_string().as_str()) {
            self.push(span, "rust-panic-prone-call", format!("production code calls {method}"));
        }
    }

    fn item_function(&mut self, item: &ItemFn) {
        self.assess_function(FunctionDetails {
            name: &item.sig.ident,
            input_count: positional_input_count(&item.sig.inputs),
            body: &item.block,
            span: item.span(),
        });
    }

    fn impl_function(&mut self, item: &ImplItemFn) {
        self.assess_function(FunctionDetails {
            name: &item.sig.ident,
            input_count: positional_input_count(&item.sig.inputs),
            body: &item.block,
            span: item.span(),
        });
    }

    fn trait_function(&mut self, item: &TraitItemFn, body: &syn::Block) {
        self.assess_function(FunctionDetails {
            name: &item.sig.ident,
            input_count: positional_input_count(&item.sig.inputs),
            body,
            span: item.span(),
        });
    }

    fn closure_function(&mut self, closure: &ExprClosure) {
        let span = closure.span();
        self.check_function_lines("<closure>", span);
        self.check_parameter_count("<closure>", closure.inputs.len(), span);
        self.check_function_metrics("<closure>", closure_metrics(&closure.body), span);
    }
}

impl<'ast> Visit<'ast> for RustVisitor<'_, '_> {
    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        if is_test_only(&item.attrs) {
            return;
        }
        self.item_function(item);
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        if is_test_only(&item.attrs) {
            return;
        }
        self.impl_function(item);
        visit::visit_impl_item_fn(self, item);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast TraitItemFn) {
        if is_test_only(&item.attrs) {
            return;
        }
        if let Some(body) = &item.default {
            self.trait_function(item, body);
        }
        visit::visit_trait_item_fn(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if is_test_only(&item.attrs) {
            return;
        }
        visit::visit_item_mod(self, item);
    }

    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        self.closure_function(closure);
        visit::visit_expr_closure(self, closure);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast syn::ExprMethodCall) {
        self.inspect_panic_method(&expression.method, expression.span());
        visit::visit_expr_method_call(self, expression);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let Expr::Path(path) = expression.func.as_ref()
            && let Some(segment) = path.path.segments.last()
        {
            self.inspect_panic_method(&segment.ident, expression.span());
        }
        visit::visit_expr_call(self, expression);
    }
}

fn positional_input_count(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> usize {
    inputs.iter().filter(|input| matches!(input, FnArg::Typed(_))).count()
}

fn nonblank_line_count(source: &str, span: Span) -> usize {
    let start = span_line(span);
    let end = span.end().line.max(1);
    source
        .lines()
        .enumerate()
        .filter(|(index, line)| (start..=end).contains(&(index + 1)) && !line.trim().is_empty())
        .count()
}
