//! Task-bound capability propagation through value expressions.
//!
//! **Documentation:** `docs/pascal/language/types/channels.md`.

use super::Checker;
use crate::types::{Ty, TypeConstraint};
use fpas_parser::Expr;

impl Checker {
    /// Propagate task-bound closure state through values that can cross task boundaries.
    pub(in crate::check) fn propagate_task_bound_expr(&mut self, expr: &Expr, key: usize) {
        let task_bound = match expr {
            Expr::Paren(inner, _)
            | Expr::ResultOk(inner, _)
            | Expr::ResultError(inner, _)
            | Expr::OptionSome(inner, _)
            | Expr::Try(inner, _)
            | Expr::NamedArgument { value: inner, .. } => {
                self.expr_is_task_bound(Self::expr_lookup_key(inner))
            }
            Expr::ArrayLiteral(elements, _) => elements
                .iter()
                .any(|element| self.expr_is_task_bound(Self::expr_lookup_key(element))),
            Expr::If {
                branches,
                else_value,
                ..
            } => branches
                .iter()
                .map(|branch| &branch.value)
                .chain(std::iter::once(else_value.as_ref()))
                .any(|value| self.expr_is_task_bound(Self::expr_lookup_key(value))),
            Expr::Case { arms, else_arm, .. } => arms
                .iter()
                .map(|arm| &arm.value)
                .chain(else_arm.iter().map(|else_arm| &else_arm.value))
                .any(|value| self.expr_is_task_bound(Self::expr_lookup_key(value))),
            Expr::DictLiteral(pairs, _) => pairs.iter().any(|(key, value)| {
                self.expr_is_task_bound(Self::expr_lookup_key(key))
                    || self.expr_is_task_bound(Self::expr_lookup_key(value))
            }),
            Expr::Call { args, .. } if self.record_constructions.contains(&key) => {
                args.iter().any(|argument| {
                    self.expr_is_task_bound(Self::expr_lookup_key(argument.argument_value()))
                }) || self.expr_types.get(&key).is_some_and(|ty| {
                    let Ty::Record(record) = ty else {
                        return false;
                    };
                    self.record_defaults
                        .get(&record.name)
                        .is_some_and(|defaults| {
                            defaults.iter().any(|(name, value)| {
                                !args.iter().any(|argument| {
                                    argument
                                        .argument_name()
                                        .is_some_and(|provided| provided.eq_ignore_ascii_case(name))
                                }) && value.as_ref().is_some_and(|value| {
                                    self.expr_is_task_bound(Self::expr_lookup_key(value))
                                })
                            })
                        })
                })
            }
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

    /// Whether this value type can retain callable captures, including nested payloads.
    pub(in crate::check) fn type_can_contain_callable(&self, ty: &Ty) -> bool {
        self.type_can_contain_callable_inner(ty, &mut std::collections::HashSet::new())
    }

    fn type_can_contain_callable_inner(
        &self,
        ty: &Ty,
        visited_types: &mut std::collections::HashSet<String>,
    ) -> bool {
        match self.resolve_visible_type(ty) {
            Ty::GenericParam(_, Some(TypeConstraint::Numeric | TypeConstraint::Comparable)) => {
                false
            }
            Ty::Function(_) | Ty::Procedure(_) | Ty::GenericParam(_, _) => true,
            Ty::Array(inner) | Ty::Option(inner) => {
                self.type_can_contain_callable_inner(&inner, visited_types)
            }
            Ty::Result(ok, error) | Ty::Dict(ok, error) => {
                self.type_can_contain_callable_inner(&ok, visited_types)
                    || self.type_can_contain_callable_inner(&error, visited_types)
            }
            Ty::Record(record) => {
                let identity = format!(
                    "record:{}",
                    Ty::Record(record.clone()).to_string().to_ascii_lowercase()
                );
                if !visited_types.insert(identity.clone()) {
                    return false;
                }
                let contains_callable = record
                    .fields
                    .iter()
                    .any(|(_, field)| self.type_can_contain_callable_inner(field, visited_types));
                visited_types.remove(&identity);
                contains_callable
            }
            Ty::Enum(enumeration) => {
                let resolved = self.resolve_visible_type(&Ty::Enum(enumeration.clone()));
                let Ty::Enum(enumeration) = resolved else {
                    return false;
                };
                let identity = format!(
                    "enum:{}",
                    Ty::Enum(enumeration.clone())
                        .to_string()
                        .to_ascii_lowercase()
                );
                if !visited_types.insert(identity.clone()) {
                    return false;
                }
                let contains_callable = enumeration.variants.iter().any(|variant| {
                    variant.fields.iter().any(|(_, field)| {
                        self.type_can_contain_callable_inner(field, visited_types)
                    })
                });
                visited_types.remove(&identity);
                contains_callable
            }
            Ty::Integer
            | Ty::Real
            | Ty::Boolean
            | Ty::String
            | Ty::Distinct(_)
            | Ty::Unit
            | Ty::Channel(_)
            | Ty::Named(_)
            | Ty::Task(_)
            | Ty::Error => false,
        }
    }
}
