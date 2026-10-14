//! Evaluation of expressions, closure creation, and callable origins.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::collection::Owner;
use super::identity;
use super::walker::{Origin, Value, Walker};
use crate::check::MethodCallTarget;
use fpas_parser::{Expr, PostfixOperation};

impl Walker<'_, '_> {
    /// Evaluate dependencies and preserve possible callable origins of the result.
    pub fn expr(&mut self, expr: &Expr) -> Value {
        let value = match expr {
            Expr::Designator(designator) => {
                let value = self.designator(designator, true);
                if self
                    .checker
                    .bound_methods
                    .contains_key(&crate::designator_lookup_key(designator))
                {
                    [Origin::BoundMethod(
                        crate::designator_lookup_key(designator),
                        identity(designator.span),
                    )]
                    .into_iter()
                    .collect()
                } else {
                    value
                }
            }
            Expr::VarArgument { designator, .. } => self.designator(designator, false),
            Expr::Call {
                designator,
                args,
                span,
            } => self.call(crate::expr_lookup_key(expr), designator, args, *span),
            Expr::Closure(closure) => {
                let owner = Owner::Closure(identity(closure.span));
                self.captures(owner, closure.span);
                [Origin::Routine(owner)].into_iter().collect()
            }
            Expr::Paren(inner, _)
            | Expr::NamedArgument { value: inner, .. }
            | Expr::Try(inner, _)
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::UnaryOp { operand: inner, .. }
            | Expr::Is { value: inner, .. } => self.expr(inner),
            Expr::Go(inner, _) => {
                let previous = self.worker_start.replace(crate::expr_lookup_key(inner));
                let value = self.expr(inner);
                self.worker_start = previous;
                value
            }
            Expr::BinaryOp { left, right, .. } => self
                .expr(left)
                .into_iter()
                .chain(self.expr(right))
                .collect(),
            Expr::ArrayLiteral(values, _) => {
                values.iter().flat_map(|value| self.expr(value)).collect()
            }
            Expr::DictLiteral(values, _) => values
                .iter()
                .flat_map(|(key, value)| self.expr(key).into_iter().chain(self.expr(value)))
                .collect(),
            Expr::RecordUpdate { base, fields, .. } => {
                let mut values = self.expr(base);
                for field in fields {
                    values.extend(self.expr(&field.value));
                }
                values
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                let mut receiver = self.expr(base);
                for operation in operations {
                    match operation {
                        PostfixOperation::Field { .. } => {}
                        PostfixOperation::Index { index, .. } => {
                            self.expr(index);
                        }
                        PostfixOperation::MethodCall { args, span, .. } => {
                            let key = crate::postfix_operation_lookup_key(operation);
                            let mut values: Vec<_> = args
                                .iter()
                                .map(|arg| self.expr(arg.argument_value()))
                                .collect();
                            self.order_arguments(args, &mut values);
                            self.invalidate_var_arguments(args);
                            let previous = self.worker;
                            if self.worker_start == Some(crate::expr_lookup_key(expr))
                                && operations
                                    .last()
                                    .is_some_and(|last| std::ptr::eq(operation, last))
                            {
                                self.worker = true;
                            }
                            receiver = if let Some(target) = self.checker.method_calls.get(&key) {
                                if matches!(target, MethodCallTarget::Instance { .. }) {
                                    values.insert(0, receiver);
                                }
                                self.method(target.qualified_name(), &values, *span)
                            } else if self.checker.fluent_calls.contains_key(&key) {
                                let target = &self.checker.fluent_calls[&key];
                                self.standard(&target.name, &values, receiver, *span)
                            } else {
                                self.invoke(&receiver, &values, *span)
                            };
                            self.worker = previous;
                        }
                    }
                }
                receiver
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                let mut values = Value::new();
                for branch in branches {
                    self.expr(&branch.condition);
                    values.extend(self.expr(&branch.value));
                }
                values.extend(self.expr(else_value));
                values
            }
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => {
                let origins = self.expr(selector);
                let mut values = Value::new();
                for arm in arms {
                    self.values.insert(identity(arm.span), origins.clone());
                    self.bind_pattern_values(&arm.labels, &origins);
                    self.labels(&arm.labels);
                    if let Some(guard) = &arm.guard {
                        self.expr(guard);
                    }
                    values.extend(self.expr(&arm.value));
                }
                if let Some(arm) = else_arm {
                    values.extend(self.expr(&arm.value));
                }
                values
            }
            _ => Value::new(),
        };
        self.filtered(expr, value)
    }
}
