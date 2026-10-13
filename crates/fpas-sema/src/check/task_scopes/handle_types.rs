//! Cycle-safe type filtering for scope-owned handle escape checks.
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use crate::check::Checker;
use crate::types::{Ty, TypeConstraint};
use std::collections::HashSet;

impl Checker {
    /// Whether a declared value type may contain a handle, including callable captures.
    pub(in crate::check) fn may_hold_task_handle(&self, ty: &Ty) -> bool {
        self.type_may_hold_task(ty, &mut HashSet::new())
    }

    fn type_may_hold_task(&self, ty: &Ty, seen: &mut HashSet<String>) -> bool {
        match ty {
            Ty::Task(_) | Ty::Function(_) | Ty::Procedure(_) => true,
            Ty::GenericParam(_, Some(TypeConstraint::Numeric | TypeConstraint::Comparable)) => {
                false
            }
            Ty::GenericParam(..) => true,
            Ty::Channel(inner) if inner.is_error() => true,
            Ty::Array(inner) | Ty::Channel(inner) | Ty::Option(inner) => {
                self.type_may_hold_task(inner, seen)
            }
            Ty::Dict(key, value) | Ty::Result(key, value) => {
                self.type_may_hold_task(key, seen) || self.type_may_hold_task(value, seen)
            }
            Ty::Named(name) => {
                let key = name.to_ascii_lowercase();
                if !seen.insert(key.clone()) {
                    return false;
                }
                let result = self
                    .scopes
                    .lookup_type(name)
                    .is_none_or(|symbol| self.type_may_hold_task(&symbol.ty, seen));
                seen.remove(&key);
                result
            }
            Ty::Record(_) | Ty::Enum(_) => {
                let resolved = self.resolve_visible_type(ty);
                let key = resolved.to_string().to_ascii_lowercase();
                if !seen.insert(key.clone()) {
                    return false;
                }
                let result = match &resolved {
                    Ty::Record(record) => record
                        .fields
                        .iter()
                        .any(|(_, field)| self.type_may_hold_task(field, seen)),
                    Ty::Enum(enumeration) => enumeration.variants.iter().any(|variant| {
                        variant
                            .fields
                            .iter()
                            .any(|(_, field)| self.type_may_hold_task(field, seen))
                    }),
                    _ => false,
                };
                seen.remove(&key);
                result
            }
            _ => false,
        }
    }
}
