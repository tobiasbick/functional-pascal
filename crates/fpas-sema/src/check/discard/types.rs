//! Cycle-safe checks of declared types, including inactive aggregate payloads.
//!
//! **Documentation:** `docs/pascal/language/functions/discard.md`

use super::Checker;
use crate::types::{Ty, TypeConstraint};
use std::collections::HashSet;

/// Whether a declared type is safe, needs capture information, or forbids discard.
pub(super) enum TaskSafety {
    Safe,
    Captures,
    Forbidden(String),
}

impl TaskSafety {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Forbidden(reason), _) | (_, Self::Forbidden(reason)) => Self::Forbidden(reason),
            (Self::Captures, _) | (_, Self::Captures) => Self::Captures,
            _ => Self::Safe,
        }
    }
}

impl Checker {
    /// Inspects all stored payload types with cycle protection.
    pub(super) fn task_safety(&self, ty: &Ty) -> TaskSafety {
        self.task_safety_inner(ty, &mut HashSet::new(), "value")
    }

    fn task_safety_inner(&self, ty: &Ty, visited: &mut HashSet<String>, path: &str) -> TaskSafety {
        match ty {
            Ty::Task(_) => TaskSafety::Forbidden(format!("{path} contains a task handle")),
            Ty::GenericParam(_, Some(TypeConstraint::Numeric | TypeConstraint::Comparable)) => {
                TaskSafety::Safe
            }
            Ty::GenericParam(name, _) => TaskSafety::Forbidden(format!(
                "{path} uses generic parameter `{}` whose constraint does not exclude task handles",
                crate::types::parameter_name(name)
            )),
            Ty::Function(_) | Ty::Procedure(_) => TaskSafety::Captures,
            Ty::Channel(inner) if inner.is_error() => {
                TaskSafety::Forbidden(format!("{path} has an unknown channel element type"))
            }
            Ty::Array(inner) | Ty::Channel(inner) | Ty::Option(inner) => {
                self.task_safety_inner(inner, visited, &format!("{path} element/payload"))
            }
            Ty::Dict(key, value) | Ty::Result(key, value) => {
                let left = self.task_safety_inner(key, visited, &format!("{path} key/success"));
                left.combine(self.task_safety_inner(value, visited, &format!("{path} value/error")))
            }
            Ty::Named(name) => {
                if !visited.insert(name.to_ascii_lowercase()) {
                    return TaskSafety::Safe;
                }
                let result = self
                    .scopes
                    .lookup_type(name)
                    .map(|symbol| self.task_safety_inner(&symbol.ty, visited, path))
                    .unwrap_or_else(|| {
                        TaskSafety::Forbidden(format!("{path} has unresolved type `{name}`"))
                    });
                visited.remove(&name.to_ascii_lowercase());
                result
            }
            Ty::Record(record) => {
                // Recursive field types can retain the empty descriptor installed
                // before the record's fields were checked. Inspect the completed
                // declaration while preserving already resolved field types.
                let resolved = self.resolve_visible_type(&Ty::Record(record.clone()));
                let Ty::Record(record) = resolved else {
                    return TaskSafety::Safe;
                };
                let key = format!(
                    "record:{}",
                    Ty::Record(record.clone()).to_string().to_ascii_lowercase()
                );
                if !visited.insert(key.clone()) {
                    return TaskSafety::Safe;
                }
                let result =
                    record
                        .fields
                        .iter()
                        .fold(TaskSafety::Safe, |safety, (name, field)| {
                            safety.combine(self.task_safety_inner(
                                field,
                                visited,
                                &format!("{path}.{name}"),
                            ))
                        });
                visited.remove(&key);
                result
            }
            Ty::Enum(enumeration) => {
                let resolved = self.resolve_visible_type(&Ty::Enum(enumeration.clone()));
                let Ty::Enum(enumeration) = resolved else {
                    return TaskSafety::Safe;
                };
                let key = format!(
                    "enum:{}",
                    Ty::Enum(enumeration.clone())
                        .to_string()
                        .to_ascii_lowercase()
                );
                if !visited.insert(key.clone()) {
                    return TaskSafety::Safe;
                }
                let result =
                    enumeration
                        .variants
                        .iter()
                        .fold(TaskSafety::Safe, |safety, variant| {
                            variant.fields.iter().fold(safety, |safety, (name, field)| {
                                safety.combine(self.task_safety_inner(
                                    field,
                                    visited,
                                    &format!("{path}.{}.{name}", variant.name),
                                ))
                            })
                        });
                visited.remove(&key);
                result
            }
            Ty::Integer
            | Ty::Real
            | Ty::Boolean
            | Ty::String
            | Ty::Distinct(_)
            | Ty::Unit
            | Ty::Error => TaskSafety::Safe,
        }
    }
}
