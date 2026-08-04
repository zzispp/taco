use syn::{
    BinOp, Block, Expr, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprWhile,
    visit::{self, Visit},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionMetrics {
    pub max_nesting: usize,
    pub cyclomatic_complexity: usize,
}

pub fn function_metrics(block: &Block) -> FunctionMetrics {
    let mut collector = MetricCollector {
        cyclomatic_complexity: 1,
        ..MetricCollector::default()
    };
    collector.visit_block(block);
    collector.into_metrics()
}

pub fn closure_metrics(body: &Expr) -> FunctionMetrics {
    let mut collector = MetricCollector {
        cyclomatic_complexity: 1,
        ..MetricCollector::default()
    };
    collector.visit_expr(body);
    collector.into_metrics()
}

#[derive(Default)]
struct MetricCollector {
    current_nesting: usize,
    max_nesting: usize,
    cyclomatic_complexity: usize,
}

impl MetricCollector {
    fn into_metrics(self) -> FunctionMetrics {
        FunctionMetrics {
            max_nesting: self.max_nesting,
            cyclomatic_complexity: self.cyclomatic_complexity,
        }
    }

    fn branch(&mut self, visit: impl FnOnce(&mut Self)) {
        self.current_nesting += 1;
        self.max_nesting = self.max_nesting.max(self.current_nesting);
        visit(self);
        self.current_nesting -= 1;
    }

    fn inspect_branch(&mut self, weight: usize, visit: impl FnOnce(&mut Self)) {
        self.cyclomatic_complexity += weight;
        self.branch(visit);
    }
}

impl<'ast> Visit<'ast> for MetricCollector {
    fn visit_expr_if(&mut self, expression: &'ast ExprIf) {
        self.inspect_branch(1, |collector| visit::visit_expr_if(collector, expression));
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast ExprForLoop) {
        self.inspect_branch(1, |collector| visit::visit_expr_for_loop(collector, expression));
    }

    fn visit_expr_while(&mut self, expression: &'ast ExprWhile) {
        self.inspect_branch(1, |collector| visit::visit_expr_while(collector, expression));
    }

    fn visit_expr_loop(&mut self, expression: &'ast ExprLoop) {
        self.inspect_branch(1, |collector| visit::visit_expr_loop(collector, expression));
    }

    fn visit_expr_match(&mut self, expression: &'ast ExprMatch) {
        let weight = expression.arms.len().saturating_sub(1);
        self.inspect_branch(weight, |collector| visit::visit_expr_match(collector, expression));
    }

    fn visit_expr_binary(&mut self, expression: &'ast ExprBinary) {
        if matches!(expression.op, BinOp::And(_) | BinOp::Or(_)) {
            self.cyclomatic_complexity += 1;
        }
        visit::visit_expr_binary(self, expression);
    }

    fn visit_expr_closure(&mut self, _expression: &'ast syn::ExprClosure) {
        // Nested closures are assessed independently by the source visitor.
    }

    fn visit_expr(&mut self, expression: &'ast Expr) {
        visit::visit_expr(self, expression);
    }
}
