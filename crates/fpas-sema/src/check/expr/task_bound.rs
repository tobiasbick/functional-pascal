//! Task-bound capability propagation through value expressions.
//!
//! **Documentation:** `docs/pascal/language/types/channels.md`.

use super::Checker;
use crate::types::Ty;
use fpas_parser::Expr;

impl Checker {
    /// Propagate task-bound closure state through values that can cross task boundaries.
    pub(in crate::check) fn propagate_task_bound_expr(&mut self, expr: &Expr, key: usize) {
        let task_bound = match expr {
            Expr::If(decision) => std::iter::once(&decision.then_value)
                .chain(decision.elsif_values.iter().map(|(_, value)| value))
                .chain(std::iter::once(&decision.else_value))
                .any(|value| self.expr_is_task_bound(Self::expr_lookup_key(value))),
            Expr::Case(decision) => decision
                .arms
                .iter()
                .map(|arm| &arm.body)
                .chain(decision.else_value.iter())
                .any(|value| self.expr_is_task_bound(Self::expr_lookup_key(value))),
            Expr::Paren(inner, _)
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _) => self.expr_is_task_bound(Self::expr_lookup_key(inner)),
            Expr::ArrayLiteral(elements, _) => elements
                .iter()
                .any(|element| self.expr_is_task_bound(Self::expr_lookup_key(element))),
            Expr::DictLiteral(pairs, _) => pairs.iter().any(|(key, value)| {
                self.expr_is_task_bound(Self::expr_lookup_key(key))
                    || self.expr_is_task_bound(Self::expr_lookup_key(value))
            }),
            Expr::RecordConstruction { fields, .. } => fields
                .iter()
                .any(|field| self.expr_is_task_bound(Self::expr_lookup_key(&field.value))),
            Expr::RecordUpdate { base, fields, .. } => {
                self.expr_is_task_bound(Self::expr_lookup_key(base))
                    || fields
                        .iter()
                        .any(|field| self.expr_is_task_bound(Self::expr_lookup_key(&field.value)))
            }
            Expr::Postfix { base, .. } => {
                self.expr_is_task_bound(Self::expr_lookup_key(base))
                    && self
                        .expr_types
                        .get(&key)
                        .is_some_and(|ty| self.type_can_contain_callable(ty))
            }
            Expr::Designator(designator) => {
                self.designator_refers_to_task_bound(designator)
                    && self
                        .expr_types
                        .get(&key)
                        .is_some_and(|ty| self.type_can_contain_callable(ty))
            }
            _ => false,
        };
        if task_bound {
            self.mark_expr_task_bound(key);
        }
    }

    /// Whether a value type can carry callable capture state through a postfix chain.
    pub(in crate::check) fn type_can_contain_callable(&self, ty: &Ty) -> bool {
        ty.contains_callable_with(|ty| self.resolve_visible_type(ty))
    }
}
