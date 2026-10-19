//! Exact function-and-binding admission for scalar local trace selectors.

use super::{DebugTraceIdentity, DebugTraceWatch, binding};
use fpas_bytecode::{DebugBindingKind, DebugType, Executable};

pub(super) fn bind(image: &Executable, selector: &str) -> Result<DebugTraceWatch, String> {
    let Some((owner, name)) = selector.split_once("::") else {
        unreachable!("local selector delimiter checked");
    };
    if owner.is_empty() || !binding::identifier(name) {
        return Err("Local trace selectors use exact executable Routine::Name, without fields, indexes or expressions.".into());
    }
    let mut candidate = None;
    for (function_id, function) in image.functions.iter().enumerate() {
        let Some(function_name) = image
            .strings
            .get(function.name)
            .filter(|n| n.eq_ignore_ascii_case(owner))
        else {
            continue;
        };
        for (binding_id, local) in function.debug.bindings.iter().enumerate() {
            if local.hidden
                || local.kind == DebugBindingKind::Capture
                || !image
                    .strings
                    .get(local.name)
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
            {
                continue;
            }
            if candidate.is_some() {
                return Err(format!(
                    "Trace `{selector}` is ambiguous across functions or lexical bindings; use a uniquely named binding in the full executable routine name."
                ));
            }
            let ty = match image.debug_types.get(local.ty.get() as usize) {
                Some(DebugType::Reference(inner)) => *inner,
                _ => local.ty,
            };
            if binding::scalar_type(image, ty).is_none()
                || (local.kind == DebugBindingKind::Local && local.initializer.is_none())
            {
                return Err(format!(
                    "Trace `{selector}` requires a scalar parameter or a local with exact declaration-store metadata."
                ));
            }
            if local.kind == DebugBindingKind::Parameter
                && function
                    .debug
                    .bindings
                    .iter()
                    .filter(|binding| binding.kind == DebugBindingKind::Parameter)
                    .count()
                    != usize::from(function.arity)
            {
                return Err("Local trace requires complete parameter admission metadata.".into());
            }
            let type_name = image
                .strings
                .get(local.type_name)
                .ok_or("Missing local type metadata")?;
            let name = image
                .strings
                .get(local.name)
                .ok_or("Missing local name metadata")?;
            if type_name.len() > 4096
                || function_name
                    .len()
                    .saturating_add(name.len())
                    .saturating_add(2)
                    > 4096
            {
                return Err("Local trace metadata exceeds 4096 UTF-8 bytes.".into());
            }
            candidate = Some(DebugTraceWatch {
                id: 0,
                request: selector.into(),
                name: format!("{function_name}::{name}"),
                identity: DebugTraceIdentity::Local {
                    function_id: u16::try_from(function_id)
                        .map_err(|_| "Trace function identity exceeds the executable ID range")?,
                    binding_id,
                    scope_id: local.scope,
                    kind: if local.kind == DebugBindingKind::Parameter {
                        "parameter"
                    } else {
                        "local"
                    },
                },
                type_name: type_name.trim_start_matches("var ").into(),
                ty,
            });
        }
    }
    candidate.ok_or_else(|| format!("Trace `{selector}` requires one visible scalar local or parameter in an exact executable routine; captures and function aliases are unsupported."))
}
