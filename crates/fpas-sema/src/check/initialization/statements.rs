//! Ordered bindings and conservative branch/loop evaluation for initialization checks.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::identity;
use super::walker::{Origin, Walker};
use fpas_parser::Stmt;

impl Walker<'_, '_> {
    /// Check executable statements in their original initialization order.
    pub fn statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Const(binding) | Stmt::Var(binding) => {
                let value = self.expr(&binding.value);
                self.values.insert(identity(binding.span), value);
                self.ready.insert(identity(binding.span));
            }
            Stmt::Block(body, _) | Stmt::Scope { body, .. } => self.statements(body),
            Stmt::Return(Some(value), _) => {
                let returned = self.expr(value);
                self.summary.returned.extend(returned);
            }
            Stmt::Panic(value, _)
            | Stmt::Discard { expr: value, .. }
            | Stmt::Expression { expr: value, .. } => {
                self.expr(value);
            }
            Stmt::Assign { target, value, .. } => {
                self.designator(target, false);
                let mut origins = self.expr(value);
                origins.insert(Origin::Unknown);
                if let Some(binding) = self
                    .checker
                    .initialization
                    .references
                    .get(&identity(target.span))
                    .and_then(|reference| reference.declaration)
                {
                    self.write(binding);
                    self.values.insert(binding, origins);
                }
            }
            Stmt::Call {
                designator,
                args,
                span,
            } => {
                self.call(
                    crate::designator_lookup_key(designator),
                    designator,
                    args,
                    *span,
                );
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expr(condition);
                self.branch(|walk| walk.statement(then_branch));
                if let Some(branch) = else_branch {
                    self.branch(|walk| walk.statement(branch));
                }
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                ..
            } => {
                let origins = self.expr(expr);
                for arm in arms {
                    self.branch(|walk| {
                        walk.values.insert(identity(arm.span), origins.clone());
                        walk.bind_pattern_values(&arm.labels, &origins);
                        walk.labels(&arm.labels);
                        if let Some(guard) = &arm.guard {
                            walk.expr(guard);
                        }
                        walk.statement(&arm.body);
                    });
                }
                if let Some(body) = else_body {
                    self.branch(|walk| walk.statements(body));
                }
            }
            Stmt::For {
                start, end, body, ..
            } => {
                self.expr(start);
                self.expr(end);
                self.branch(|walk| walk.statement(body));
            }
            Stmt::ForIn {
                iterable,
                body,
                span,
                ..
            } => {
                let origins = self.expr(iterable);
                self.branch(|walk| {
                    walk.values.insert(identity(*span), origins);
                    walk.ready.insert(identity(*span));
                    walk.statement(body);
                });
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.expr(condition);
                self.branch(|walk| walk.statement(body));
            }
            Stmt::Repeat {
                body, condition, ..
            } => {
                self.branch(|walk| {
                    walk.statements(body);
                    walk.expr(condition);
                });
            }
            _ => {}
        }
    }

    fn branch(&mut self, evaluate: impl FnOnce(&mut Self)) {
        let ready = self.ready.clone();
        let values = self.values.clone();
        evaluate(self);
        self.ready = ready;
        for (binding, before) in values {
            self.values.entry(binding).or_default().extend(before);
        }
    }

    /// Preserve callable origins through every payload binding before checking its arm.
    pub(super) fn bind_pattern_values(
        &mut self,
        labels: &[fpas_parser::CaseLabel],
        origins: &super::walker::Value,
    ) {
        for label in labels {
            if let fpas_parser::CaseLabel::Pattern(pattern) = label {
                self.bind_pattern_value(pattern, origins);
            }
        }
    }

    fn bind_pattern_value(
        &mut self,
        pattern: &fpas_parser::Pattern,
        origins: &super::walker::Value,
    ) {
        match pattern {
            fpas_parser::Pattern::Binding { span, .. } => {
                self.values.insert(identity(*span), origins.clone());
                self.ready.insert(identity(*span));
            }
            fpas_parser::Pattern::Variant { fields, .. }
            | fpas_parser::Pattern::Destructure { fields, .. } => {
                for field in fields {
                    self.bind_pattern_value(&field.pattern, origins);
                }
            }
            _ => {}
        }
    }

    /// Account for named compile-time values used in labels and nested payload patterns.
    pub fn labels(&mut self, labels: &[fpas_parser::CaseLabel]) {
        for label in labels {
            match label {
                fpas_parser::CaseLabel::Value { start, end, .. } => {
                    self.expr(start);
                    if let Some(end) = end {
                        self.expr(end);
                    }
                }
                fpas_parser::CaseLabel::Pattern(pattern) => self.pattern(pattern),
                _ => {}
            }
        }
    }

    fn pattern(&mut self, pattern: &fpas_parser::Pattern) {
        match pattern {
            fpas_parser::Pattern::Value(value) => {
                self.expr(value);
            }
            fpas_parser::Pattern::Variant { fields, .. }
            | fpas_parser::Pattern::Destructure { fields, .. } => {
                for field in fields {
                    self.pattern(&field.pattern);
                }
            }
            _ => {}
        }
    }
}
