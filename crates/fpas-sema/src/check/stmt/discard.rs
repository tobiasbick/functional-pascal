//! Explicit result consumption and task-handle protection.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::collections::HashSet;
use std::sync::Arc;

impl Checker {
    /// Require an explicit consumer for a function call used as a statement.
    pub(in crate::check) fn require_consumed_call_result(&mut self, ty: &Ty, span: Span) {
        if !ty.is_error() && *ty != Ty::Unit {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Function result must be consumed",
                "Store or use the result, or write `discard Function(...)`.",
                span,
            );
        }
    }

    /// Check a discarded value, including task handles nested in aggregate types.
    pub(super) fn check_discard_stmt(&mut self, expr: &Expr, span: Span) {
        let ty = self.check_expr(expr);
        if self.discard_type_contains_task(&ty, &mut HashSet::new()) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Cannot discard a task handle or a value containing task handles",
                "Retain the task handle and wait for its result explicitly.",
                span,
            );
        } else if ty == Ty::Unit {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Discard requires a value",
                "Call a procedure directly as a statement; it produces no value.",
                span,
            );
        }
    }

    fn discard_type_contains_task(&self, ty: &Ty, visited: &mut HashSet<usize>) -> bool {
        match self.resolve_visible_type(ty) {
            Ty::Task(_) => true,
            Ty::Array(inner) | Ty::Option(inner) | Ty::Channel(inner) => {
                self.discard_type_contains_task(&inner, visited)
            }
            Ty::Dict(key, value) | Ty::Result(key, value) => {
                self.discard_type_contains_task(&key, visited)
                    || self.discard_type_contains_task(&value, visited)
            }
            Ty::Record(record) => {
                if !visited.insert(Arc::as_ptr(&record) as usize) {
                    return false;
                }
                record
                    .fields
                    .iter()
                    .any(|(_, field)| self.discard_type_contains_task(field, visited))
            }
            Ty::Enum(enumeration) => {
                if !visited.insert(Arc::as_ptr(&enumeration) as usize) {
                    return false;
                }
                enumeration.variants.iter().any(|variant| {
                    variant
                        .fields
                        .iter()
                        .any(|(_, field)| self.discard_type_contains_task(field, visited))
                })
            }
            _ => false,
        }
    }
}
