//! Exact function-and-binding admission for scalar local trace selectors.

use super::{DebugTraceIdentity, DebugTraceWatch};
use crate::{DebugBindingKind, DebugProjectionKind, DebugType, Executable};

pub(super) fn bind(image: &Executable, selector: &str) -> Result<DebugTraceWatch, String> {
    let Some((owner, path)) = selector.split_once("::") else {
        unreachable!("local selector delimiter checked");
    };
    let (name, parts) = super::selector::local(path)?;
    if owner.is_empty() {
        return Err("Local traces require exact Routine::Binding.".into());
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
        let direct = function.debug.bindings.iter().any(|b| {
            !b.hidden
                && b.kind != DebugBindingKind::Capture
                && image
                    .strings
                    .get(b.name)
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
        });
        let mut roots = Vec::new();
        for (binding_id, local) in function.debug.bindings.iter().enumerate() {
            if local.hidden
                || (direct && local.kind == DebugBindingKind::Capture)
                || !image
                    .strings
                    .get(local.name)
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
            {
                continue;
            }
            let capture = if local.kind == DebugBindingKind::Capture {
                let Some(capture) =
                    super::capture_binding::resolve(image, function_id, binding_id)?
                else {
                    continue;
                };
                Some(capture)
            } else {
                None
            };
            roots.push((binding_id, local, capture));
        }
        // Compare exact retained provenance only. An unrelated owner block local
        // cannot hide a capture. Equal nearest declarations remain ambiguous.
        let nearest = roots
            .iter()
            .filter_map(|(_, _, c)| c.as_ref().map(|c| c.depth))
            .min();
        for (binding_id, local, capture) in roots
            .into_iter()
            .filter(|(_, _, capture)| capture.as_ref().is_none_or(|c| Some(c.depth) == nearest))
        {
            if candidate.is_some() {
                return Err(format!(
                    "Trace `{selector}` is ambiguous across functions or lexical bindings; use a uniquely named binding in the full executable routine name."
                ));
            }
            let ty = match image.debug_types.get(local.ty.get() as usize) {
                Some(DebugType::Reference(inner)) => *inner,
                _ => local.ty,
            };
            let projection = match local.projection.as_ref() {
                Some(node) => match &node.kind {
                    DebugProjectionKind::Reference(inner) => Some(inner.as_ref()),
                    DebugProjectionKind::Operational => None,
                    _ => Some(node),
                },
                None => None,
            };
            if local.kind == DebugBindingKind::Local && local.initializer.is_none() {
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
            let type_name = capture
                .as_ref()
                .map(|c| c.type_name)
                .or_else(|| image.strings.get(local.type_name))
                .ok_or("Missing local type metadata")?;
            let fields = super::stored_path::bind(
                image,
                ty,
                Some(type_name.trim_start_matches("var ")),
                projection,
                &parts,
            )?;
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
                name: format!("{function_name}::{name}{}", fields.suffix),
                identity: DebugTraceIdentity::Local {
                    function_id: u16::try_from(function_id)
                        .map_err(|_| "Trace function identity exceeds the executable ID range")?,
                    binding_id,
                    scope_id: local.scope,
                    field_slots: fields.slots,
                    path: if fields.indexed {
                        fields.steps.clone()
                    } else {
                        Vec::new()
                    },
                    kind: match local.kind {
                        DebugBindingKind::Parameter => "parameter",
                        DebugBindingKind::Local => "local",
                        DebugBindingKind::Capture => "capture",
                    },
                    capture_source: capture.map(|c| c.source),
                },
                type_name: fields.type_name,
                ty: fields.ty,
                path_types: fields.path_types,
                path: fields.steps,
            });
        }
    }
    candidate.ok_or_else(|| format!("Trace `{selector}` requires one visible scalar local, parameter or capture in an exact executable routine; function aliases are unsupported."))
}
