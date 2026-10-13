//! Discover callable bodies nested inside retained expressions.
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`.

use super::{Binding, Model, Owner, Routine};
use crate::check::initialization::identity;
use fpas_parser::{Expr, PostfixOperation};

impl<'a> Model<'a> {
    /// Retain anonymous routine ownership and nested callable expressions.
    pub(super) fn expression(&mut self, expr: &'a Expr, owner: Owner) {
        match expr {
            Expr::Closure(closure) => {
                let key = Owner::Closure(identity(closure.span));
                self.routines.insert(
                    key,
                    Routine {
                        body: &closure.body,
                        params: &closure.params,
                        requires: &[],
                        ensures: &[],
                        capture_key: crate::expr_lookup_key(expr),
                        export_name: None,
                    },
                );
                for (index, param) in closure.params.iter().enumerate() {
                    self.bindings.insert(
                        identity(param.span),
                        Binding {
                            name: param.name.clone(),
                            owner: key,
                            initializer: None,
                            mutable: param.mode == fpas_parser::ParamMode::Var,
                            parameter: Some(index),
                            startup: false,
                        },
                    );
                }
                self.body(&closure.body, key);
            }
            Expr::Call {
                designator, args, ..
            } => {
                self.designator(designator, owner);
                for arg in args {
                    self.expression(arg, owner);
                }
            }
            Expr::Designator(designator) | Expr::VarArgument { designator, .. } => {
                self.designator(designator, owner)
            }
            Expr::Paren(inner, _)
            | Expr::UnaryOp { operand: inner, .. }
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _)
            | Expr::Go(inner, _)
            | Expr::NamedArgument { value: inner, .. }
            | Expr::Is { value: inner, .. } => self.expression(inner, owner),
            Expr::BinaryOp { left, right, .. } => {
                self.expression(left, owner);
                self.expression(right, owner);
            }
            Expr::ArrayLiteral(values, _) => {
                for value in values {
                    self.expression(value, owner);
                }
            }
            Expr::DictLiteral(values, _) => {
                for (key, value) in values {
                    self.expression(key, owner);
                    self.expression(value, owner);
                }
            }
            Expr::RecordUpdate { base, fields, .. } => {
                self.expression(base, owner);
                for field in fields {
                    self.expression(&field.value, owner);
                }
            }
            Expr::Postfix {
                base, operations, ..
            } => {
                self.expression(base, owner);
                for operation in operations {
                    match operation {
                        PostfixOperation::Index { index, .. } => self.expression(index, owner),
                        PostfixOperation::MethodCall { args, .. } => {
                            for arg in args {
                                self.expression(arg, owner);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Expr::If {
                branches,
                else_value,
                ..
            } => {
                for branch in branches {
                    self.expression(&branch.condition, owner);
                    self.expression(&branch.value, owner);
                }
                self.expression(else_value, owner);
            }
            Expr::Case {
                selector,
                arms,
                else_arm,
                ..
            } => {
                self.expression(selector, owner);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.expression(guard, owner);
                    }
                    self.expression(&arm.value, owner);
                }
                if let Some(arm) = else_arm {
                    self.expression(&arm.value, owner);
                }
            }
            _ => {}
        }
    }
}
