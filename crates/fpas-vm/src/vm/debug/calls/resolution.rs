//! Exact named-target resolution for routines, Std intrinsics, and enum constructors.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, Intrinsic, RuntimeEnumLayout, VerifiedExecutable};

use super::detach::error;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::layouts::RuntimeLayouts;

/// One exact named sandbox target resolved from verified executable metadata.
pub(super) enum NamedTarget {
    /// Exact executable function.
    Function(FunctionId),
    /// Fully qualified data-enum constructor.
    EnumConstructor(Arc<RuntimeEnumLayout>),
    /// Exact non-overloaded Std intrinsic.
    Intrinsic(Intrinsic),
}

/// Resolve one named debugger call to a function, enum constructor, or intrinsic.
pub(super) fn resolve_named(
    executable: &VerifiedExecutable,
    layouts: &RuntimeLayouts,
    name: &str,
) -> Result<NamedTarget, DebugSessionError> {
    let functions = matching_functions(executable, name);
    match functions.as_slice() {
        [function] => return Ok(NamedTarget::Function(*function)),
        [] => {}
        _ => {
            return Err(error(
                DebugErrorKind::AmbiguousCallable,
                format!("debug callable `{name}` has multiple exact executable targets"),
                "Use a fully qualified callable name.",
            ));
        }
    }
    let intrinsics = matching_intrinsics(name);
    match intrinsics.as_slice() {
        [intrinsic] => return Ok(NamedTarget::Intrinsic(*intrinsic)),
        [] => {}
        _ => {
            return Err(error(
                DebugErrorKind::AmbiguousCallable,
                format!("debug intrinsic `{name}` requires a statically known overload"),
                "Use a non-overloaded intrinsic in debugger evaluation.",
            ));
        }
    }
    match matching_enum_constructor(layouts, name) {
        ConstructorMatch::Exact(layout) => Ok(NamedTarget::EnumConstructor(layout)),
        ConstructorMatch::Ambiguous => Err(error(
            DebugErrorKind::AmbiguousCallable,
            format!("debug enum constructor `{name}` matches multiple executable variants"),
            "Use a fully qualified Type.Variant name that identifies one enum variant.",
        )),
        ConstructorMatch::UnknownVariant { owner } => Err(error(
            DebugErrorKind::UnknownCallable,
            format!("debug enum `{owner}` has no variant matching `{name}`"),
            "Use a fully qualified Type.Variant name from the executable metadata.",
        )),
        ConstructorMatch::None => Err(error(
            DebugErrorKind::UnknownCallable,
            format!("debug callable `{name}` is not present in the executable catalog"),
            "Use an exact named routine, fully qualified enum constructor, fully qualified Std intrinsic, or visible function value.",
        )),
    }
}

fn matching_functions(executable: &VerifiedExecutable, name: &str) -> Vec<FunctionId> {
    executable
        .executable()
        .functions
        .iter()
        .enumerate()
        .filter(|(_, function)| {
            executable
                .executable()
                .strings
                .get(function.name)
                .is_some_and(|candidate| callable_name_matches(candidate, name))
        })
        .map(|(index, _)| FunctionId::new(u16::try_from(index).unwrap_or(u16::MAX)))
        .collect()
}

fn matching_intrinsics(name: &str) -> Vec<Intrinsic> {
    Intrinsic::all()
        .filter(|intrinsic| callable_name_matches(&intrinsic.debugger_name(), name))
        .collect()
}

enum ConstructorMatch {
    Exact(Arc<RuntimeEnumLayout>),
    Ambiguous,
    UnknownVariant { owner: String },
    None,
}

fn matching_enum_constructor(layouts: &RuntimeLayouts, name: &str) -> ConstructorMatch {
    let Some((owner, variant)) = split_qualified_constructor(name) else {
        return ConstructorMatch::None;
    };
    let owner_matched = layouts
        .enum_variants
        .iter()
        .any(|layout| layout.type_name.eq_ignore_ascii_case(owner));
    let matches = layouts
        .enum_variants
        .iter()
        .filter(|layout| {
            layout.type_name.eq_ignore_ascii_case(owner)
                && layout.variant.eq_ignore_ascii_case(variant)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [layout] => ConstructorMatch::Exact(Arc::clone(layout)),
        [] if owner_matched => ConstructorMatch::UnknownVariant {
            owner: owner.to_string(),
        },
        [] => ConstructorMatch::None,
        _ => ConstructorMatch::Ambiguous,
    }
}

fn split_qualified_constructor(name: &str) -> Option<(&str, &str)> {
    let (owner, variant) = name.rsplit_once('.')?;
    if owner.is_empty() || variant.is_empty() || variant.contains('.') {
        None
    } else {
        Some((owner, variant))
    }
}

fn callable_name_matches(candidate: &str, requested: &str) -> bool {
    candidate.eq_ignore_ascii_case(requested)
        || (!requested.contains('.')
            && candidate
                .rsplit_once('.')
                .is_some_and(|(_, short)| short.eq_ignore_ascii_case(requested)))
}

#[cfg(test)]
mod tests {
    use super::split_qualified_constructor;

    #[test]
    fn qualified_constructor_names_split_on_the_final_dot() {
        assert_eq!(
            split_qualified_constructor("Choice.Pair"),
            Some(("Choice", "Pair"))
        );
        assert_eq!(
            split_qualified_constructor("cHoIcE.pAiR"),
            Some(("cHoIcE", "pAiR"))
        );
        assert_eq!(split_qualified_constructor("Pair"), None);
        assert_eq!(split_qualified_constructor(".Empty"), None);
        assert_eq!(split_qualified_constructor("Choice."), None);
    }
}
