//! Expression rules: which origins a value carries, and capture by anonymous routines.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use fpas_parser::{CaseLabel, ClosureExpr, DesignatorPart, Expr, Pattern, PostfixOperation};

use super::origins::{Frame, Origin, Owners, merge};
use super::walker::Walker;
use crate::check::Checker;

impl Walker<'_, '_> {
    /// Origins of `expr`, after reporting escapes inside it.
    pub(super) fn expr(&mut self, expr: &Expr) -> Owners {
        let owners = match expr {
            Expr::Integer(..)
            | Expr::Real(..)
            | Expr::Str(..)
            | Expr::Bool(..)
            | Expr::OptionNone(_)
            | Expr::Error(_) => Vec::new(),
            Expr::Designator(designator) => {
                let mut owners = Vec::new();
                for (index, part) in designator.parts.iter().enumerate() {
                    match part {
                        DesignatorPart::Ident(name, _) if index == 0 => owners = self.read(name),
                        DesignatorPart::Index(index, _) => {
                            let _ = self.expr(index);
                        }
                        DesignatorPart::Ident(..) => {}
                    }
                }
                owners
            }
            Expr::Call {
                designator,
                args,
                span,
            } => self.designator_call(Checker::expr_lookup_key(expr), designator, args, *span),
            Expr::Paren(inner, _)
            | Expr::UnaryOp { operand: inner, .. }
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _)
            | Expr::NamedArgument { value: inner, .. } => self.expr(inner),
            Expr::BinaryOp { left, right, .. } => {
                let mut owners = self.expr(left);
                merge(&mut owners, self.expr(right));
                owners
            }
            Expr::ArrayLiteral(items, _) => {
                let mut owners = Vec::new();
                for item in items {
                    merge(&mut owners, self.expr(item));
                }
                owners
            }
            Expr::DictLiteral(entries, _) => {
                let mut owners = Vec::new();
                for (key, value) in entries {
                    merge(&mut owners, self.expr(key));
                    merge(&mut owners, self.expr(value));
                }
                owners
            }
            Expr::Go(call, _) => {
                let _ = self.expr(call);
                self.scopes
                    .last()
                    .map(|span| {
                        vec![Origin::Scope {
                            depth: self.depth(),
                            span: *span,
                        }]
                    })
                    .unwrap_or_default()
            }
            Expr::RecordUpdate { base, fields, .. } => {
                let mut owners = self.expr(base);
                for field in fields {
                    merge(&mut owners, self.expr(&field.value));
                }
                owners
            }
            Expr::Postfix {
                base, operations, ..
            } => self.postfix(base, operations),
            Expr::Closure(closure) => {
                self.closure(Checker::expr_lookup_key(expr), closure);
                Vec::new()
            }
            Expr::VarArgument { designator, .. } => designator
                .parts
                .first()
                .map(|part| match part {
                    DesignatorPart::Ident(name, _) => self.read(name),
                    DesignatorPart::Index(..) => Vec::new(),
                })
                .unwrap_or_default(),
            Expr::Is { value, .. } => {
                let _ = self.expr(value);
                Vec::new()
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                let mut owners = Vec::new();
                for branch in branches {
                    self.in_frame(|walker| {
                        let _ = walker.expr(&branch.condition);
                        merge(&mut owners, walker.expr(&branch.value));
                    });
                }
                merge(&mut owners, self.expr(else_value));
                owners
            }
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => {
                let selected = self.expr(selector);
                let mut owners = Vec::new();
                for arm in arms {
                    self.in_frame(|walker| {
                        for label in &arm.labels {
                            walker.bind_case_label(label, &selected);
                        }
                        if let Some(guard) = &arm.guard {
                            let _ = walker.expr(guard);
                        }
                        merge(&mut owners, walker.expr(&arm.value));
                    });
                }
                if let Some(arm) = else_arm {
                    merge(&mut owners, self.expr(&arm.value));
                }
                owners
            }
        };
        self.filtered(expr, owners)
    }

    fn postfix(&mut self, base: &Expr, operations: &[PostfixOperation]) -> Owners {
        let mut owners = self.expr(base);
        for operation in operations {
            match operation {
                PostfixOperation::Field { .. } => {}
                PostfixOperation::Index { index, .. } => {
                    let _ = self.expr(index);
                }
                PostfixOperation::MethodCall { args, span, .. } => {
                    owners = self.receiver_call(
                        Checker::postfix_operation_lookup_key(operation),
                        owners,
                        args,
                        *span,
                    );
                }
            }
        }
        owners
    }

    /// Bind case-label pattern names to the origins of the matched value.
    pub(super) fn bind_case_label(&mut self, label: &CaseLabel, owners: &Owners) {
        match label {
            CaseLabel::Binding { name, .. } => self.declare(name, owners.clone()),
            CaseLabel::Pattern(pattern) => self.bind_pattern(pattern, owners),
            CaseLabel::Value { start, end, .. } => {
                let _ = self.expr(start);
                if let Some(end) = end {
                    let _ = self.expr(end);
                }
            }
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern, owners: &Owners) {
        match pattern {
            Pattern::Binding { name, .. } => self.declare(name, owners.clone()),
            Pattern::Variant { fields, .. } | Pattern::Destructure { fields, .. } => {
                for field in fields {
                    self.bind_pattern(&field.pattern, owners);
                }
            }
            Pattern::Value(value) => {
                let _ = self.expr(value);
            }
            Pattern::Wildcard(_) => {}
        }
    }

    /// Reject captured tracked values, then walk the anonymous routine as its own body.
    fn closure(&mut self, key: usize, closure: &ClosureExpr) {
        if let Some(info) = self.checker.closure_infos.get(&key) {
            for capture in &info.captures {
                let owners = self.read(&capture.name);
                self.escape_all(
                    &owners,
                    closure.span,
                    &format!("capture of `{}` by an anonymous routine", capture.name),
                );
            }
        }
        let scopes = std::mem::take(&mut self.scopes);
        let escapes = std::mem::take(&mut self.escapes);
        let mut frame = Frame {
            routine_boundary: true,
            ..Frame::default()
        };
        for param in &closure.params {
            frame.bindings.insert(
                crate::scope::canonical_symbol_name(&param.name),
                super::origins::Binding {
                    outer: param.mode == fpas_parser::ParamMode::Var,
                    ..Default::default()
                },
            );
        }
        self.frames.push(frame);
        self.walk_body(&closure.body, None);
        self.frames.pop();
        self.escapes = escapes;
        self.scopes = scopes;
    }
}
