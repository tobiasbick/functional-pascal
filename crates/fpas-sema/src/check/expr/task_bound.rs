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
            Expr::RecordLiteral { fields, .. } => fields
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

    fn type_can_contain_callable(&self, ty: &Ty) -> bool {
        self.type_can_contain_callable_inner(ty, &mut std::collections::HashSet::new())
    }

    fn type_can_contain_callable_inner(
        &self,
        ty: &Ty,
        visited_records: &mut std::collections::HashSet<usize>,
    ) -> bool {
        match self.resolve_visible_type(ty) {
            Ty::Function(_) | Ty::Procedure(_) | Ty::GenericParam(_, _) => true,
            Ty::Array(inner) | Ty::Option(inner) => {
                self.type_can_contain_callable_inner(&inner, visited_records)
            }
            Ty::Result(ok, error) | Ty::Dict(ok, error) => {
                self.type_can_contain_callable_inner(&ok, visited_records)
                    || self.type_can_contain_callable_inner(&error, visited_records)
            }
            Ty::Record(record) => {
                let identity = std::sync::Arc::as_ptr(&record) as usize;
                if !visited_records.insert(identity) {
                    return false;
                }
                let contains_callable = record
                    .fields
                    .iter()
                    .any(|(_, field)| self.type_can_contain_callable_inner(field, visited_records));
                visited_records.remove(&identity);
                contains_callable
            }
            Ty::Enum(enumeration) => enumeration.variants.iter().any(|variant| {
                variant
                    .fields
                    .iter()
                    .any(|(_, field)| self.type_can_contain_callable_inner(field, visited_records))
            }),
            Ty::Integer
            | Ty::Real
            | Ty::Boolean
            | Ty::String
            | Ty::Unit
            | Ty::Channel(_)
            | Ty::Named(_)
            | Ty::Task(_)
            | Ty::Error => false,
        }
    }
}
