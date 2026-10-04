//! Transitive component capabilities of recursive nominal data.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`,
//! `docs/pascal/language/functions/first-class.md`.

use super::{Ty, TypeConstraint};
use std::collections::HashMap;

#[derive(Clone, Copy)]
enum ComponentRule {
    Equality,
    Callable,
    Task,
    Finite,
}

impl ComponentRule {
    fn combines_all(self) -> bool {
        matches!(self, Self::Equality | Self::Finite)
    }

    fn backedge(self) -> bool {
        matches!(self, Self::Equality)
    }
}

impl Ty {
    /// Check whether the type can represent a value of finite size.
    pub(crate) fn has_finite_value_with(&self, resolve: impl Fn(&Ty) -> Ty) -> bool {
        check(
            self,
            ComponentRule::Finite,
            &resolve,
            &mut ComponentStates::default(),
        )
    }
    /// Check whether any stored component can carry callable capture state.
    pub(crate) fn contains_callable_with(&self, resolve: impl Fn(&Ty) -> Ty) -> bool {
        check(
            self,
            ComponentRule::Callable,
            &resolve,
            &mut ComponentStates::default(),
        )
    }

    /// Check task handles transitively through stored value components.
    pub(crate) fn contains_task_with(&self, resolve: impl Fn(&Ty) -> Ty) -> bool {
        check(
            self,
            ComponentRule::Task,
            &resolve,
            &mut ComponentStates::default(),
        )
    }
}

/// Check structural equality with the caller's nominal reference resolver.
pub(super) fn supports_equality(ty: &Ty, resolve: impl Fn(&Ty) -> Ty) -> bool {
    check(
        ty,
        ComponentRule::Equality,
        &resolve,
        &mut ComponentStates::default(),
    )
}

type StateKey = (String, Vec<bool>);

#[derive(Default)]
struct ComponentStates {
    visiting: Vec<StateKey>,
    arguments: Vec<String>,
    decisive: HashMap<StateKey, bool>,
}

impl ComponentStates {
    fn finish(&mut self, state: StateKey, result: bool, rule: ComponentRule) {
        self.visiting.pop();
        // Only cache conclusions independent of provisional backedge answers.
        // Otherwise a finite base case discovered later could leave a stale negative.
        if result != rule.backedge() {
            self.decisive.insert(state, result);
        }
    }
}

fn check(
    ty: &Ty,
    rule: ComponentRule,
    resolve: &impl Fn(&Ty) -> Ty,
    visiting: &mut ComponentStates,
) -> bool {
    match resolve(ty) {
        Ty::Error => rule.combines_all(),
        Ty::Integer | Ty::Real | Ty::Boolean | Ty::String => rule.combines_all(),
        Ty::GenericParam(_, constraint) => match rule {
            ComponentRule::Equality => {
                constraint.is_some_and(|constraint| constraint.implies(TypeConstraint::Equatable))
            }
            ComponentRule::Callable => true,
            ComponentRule::Task => false,
            ComponentRule::Finite => true,
        },
        Ty::Function(_) | Ty::Procedure(_) => {
            matches!(rule, ComponentRule::Callable | ComponentRule::Finite)
        }
        Ty::Task(_) => matches!(rule, ComponentRule::Task | ComponentRule::Finite),
        Ty::Array(inner) | Ty::Option(inner) => {
            matches!(rule, ComponentRule::Finite) || check(&inner, rule, resolve, visiting)
        }
        Ty::Channel(inner) => {
            matches!(rule, ComponentRule::Finite)
                || (matches!(rule, ComponentRule::Task) && check(&inner, rule, resolve, visiting))
        }
        Ty::Dict(_, _) if matches!(rule, ComponentRule::Finite) => true,
        Ty::Result(ok, error) if matches!(rule, ComponentRule::Finite) => {
            check(&ok, rule, resolve, visiting) || check(&error, rule, resolve, visiting)
        }
        Ty::Dict(key, value) | Ty::Result(key, value) => {
            let left = check(&key, rule, resolve, visiting);
            if rule.combines_all() {
                left && check(&value, rule, resolve, visiting)
            } else {
                left || check(&value, rule, resolve, visiting)
            }
        }
        Ty::Record(record) => {
            if record.is_resource && matches!(rule, ComponentRule::Equality) {
                return false;
            }
            if record.is_resource && matches!(rule, ComponentRule::Finite) {
                return true;
            }
            check_fields(
                &record.name,
                &record.type_args,
                record.fields.iter().map(|(_, field)| field),
                rule,
                resolve,
                visiting,
            )
        }
        Ty::Enum(enumeration) if matches!(rule, ComponentRule::Finite) => {
            let state = state_key(
                &enumeration.name,
                &enumeration.type_args,
                rule,
                resolve,
                visiting,
            );
            if let Some(result) = visiting.decisive.get(&state) {
                return *result;
            }
            if visiting.visiting.contains(&state) {
                return false;
            }
            visiting.visiting.push(state.clone());
            let result = enumeration.variants.iter().any(|variant| {
                variant
                    .fields
                    .iter()
                    .all(|(_, field)| check(field, rule, resolve, visiting))
            });
            visiting.finish(state, result, rule);
            result
        }
        Ty::Enum(enumeration) => check_fields(
            &enumeration.name,
            &enumeration.type_args,
            enumeration
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|(_, field)| field)),
            rule,
            resolve,
            visiting,
        ),
        _ => false,
    }
}

fn check_fields<'a>(
    name: &str,
    arguments: &[Ty],
    fields: impl Iterator<Item = &'a Ty>,
    rule: ComponentRule,
    resolve: &impl Fn(&Ty) -> Ty,
    visiting: &mut ComponentStates,
) -> bool {
    // Argument capabilities form a finite state space even for growing applications
    // such as Node of (array of (T)); declaration names alone would hide changed arguments.
    let state = state_key(name, arguments, rule, resolve, visiting);
    if let Some(result) = visiting.decisive.get(&state) {
        return *result;
    }
    if visiting.visiting.contains(&state) {
        return rule.backedge();
    }
    visiting.visiting.push(state.clone());
    let mut fields = fields;
    let result = if rule.combines_all() {
        fields.all(|field| check(field, rule, resolve, visiting))
    } else {
        fields.any(|field| check(field, rule, resolve, visiting))
    };
    visiting.finish(state, result, rule);
    result
}

fn state_key(
    name: &str,
    arguments: &[Ty],
    rule: ComponentRule,
    resolve: &impl Fn(&Ty) -> Ty,
    visiting: &mut ComponentStates,
) -> (String, Vec<bool>) {
    (
        name.to_ascii_lowercase(),
        arguments
            .iter()
            .map(|argument| {
                // Deferred aliases can recur inside arguments before a nominal state
                // has been computed. Their source type identity bounds that recursion.
                let key = argument.to_string().to_ascii_lowercase();
                if visiting.arguments.contains(&key) {
                    return rule.backedge();
                }
                visiting.arguments.push(key);
                let result = check(argument, rule, resolve, visiting);
                visiting.arguments.pop();
                result
            })
            .collect(),
    )
}
