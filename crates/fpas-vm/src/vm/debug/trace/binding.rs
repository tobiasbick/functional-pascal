//! Bind only immutable metadata identities, without initializing or evaluating storage.

use super::{DebugTraceIdentity, DebugTraceWatch, model::encoded_size};
use fpas_bytecode::{DebugType, DebugTypeId, Executable};

pub(super) fn bind(
    image: &Executable,
    selectors: &[String],
) -> Result<Vec<DebugTraceWatch>, String> {
    if selectors.len() > 64 {
        return Err("At most 64 trace selectors are supported.".into());
    }
    let mut watches = Vec::<DebugTraceWatch>::new();
    for selector in selectors {
        if selector.is_empty() || selector.len() > 4096 {
            return Err("Trace selectors require 1..4096 UTF-8 bytes.".into());
        }
        if selector.contains("::") {
            let mut watch = super::local_binding::bind(image, selector)?;
            if !watches.iter().any(|prior| prior.identity == watch.identity) {
                watch.id = watches.len();
                watches.push(watch);
            }
            continue;
        }
        let mut candidate = None;
        for (index, global) in image.globals.iter().enumerate() {
            let Some(name) = image.strings.get(global.name) else {
                continue;
            };
            if !global.mutable
                || !super::selector::qualified_root(name)
                || !selector
                    .get(..name.len())
                    .is_some_and(|p| p.eq_ignore_ascii_case(name))
            {
                continue;
            }
            let rest = &selector[name.len()..];
            if !rest.is_empty() && !rest.starts_with(['.', '[']) {
                continue;
            }
            let Ok(tail) = super::selector::suffix(rest) else {
                continue;
            };
            let Ok(fields) = super::stored_path::bind(
                image,
                global.ty,
                global.type_name.and_then(|id| image.strings.get(id)),
                &tail,
            ) else {
                continue;
            };
            let canonical = format!("{name}{}", fields.suffix);
            if candidate.is_some() {
                return Err(format!(
                    "Trace `{selector}` has multiple eligible metadata bindings; select an unambiguous mutable global or stored scalar field."
                ));
            }
            candidate = Some(DebugTraceWatch {
                id: watches.len(),
                request: selector.clone(),
                name: canonical,
                identity: DebugTraceIdentity::Global {
                    global_slot: index,
                    field_slots: fields.slots,
                    path: if fields.indexed {
                        fields.steps.clone()
                    } else {
                        Vec::new()
                    },
                },
                type_name: fields.type_name,
                ty: fields.ty,
                root_ty: global.ty,
                path: fields.steps,
            });
        }
        let Some(watch) = candidate else {
            return Err(format!(
                "Trace `{selector}` requires one mutable executable global or stored scalar record field, using the full metadata unit name. Use Routine::Name for scalar locals/parameters/captures; aggregates and aliases are unsupported."
            ));
        };
        if !watches.iter().any(|prior| prior.identity == watch.identity) {
            watches.push(watch);
        }
    }
    // Leave control space for counts, reasons and the envelope's other small fields.
    if encoded_size(&watches).saturating_add(encoded_size(&selectors))
        > super::MAX_TRACE_BYTES - 4096
    {
        return Err("Trace binding metadata exceeds the 2 MiB trace budget.".into());
    }
    Ok(watches)
}

pub(super) fn scalar_type(image: &Executable, ty: DebugTypeId) -> Option<&str> {
    match image.debug_types.get(ty.get() as usize)? {
        DebugType::Integer => Some("integer"),
        DebugType::Real => Some("real"),
        DebugType::Boolean => Some("boolean"),
        DebugType::String => Some("string"),
        DebugType::Enum(id) => {
            if image
                .enum_variants
                .iter()
                .filter(|variant| variant.owner == *id)
                .any(|variant| !variant.fields.is_empty())
            {
                return None;
            }
            image
                .enums
                .get(id.get() as usize)
                .and_then(|layout| image.strings.get(layout.name))
        }
        _ => None,
    }
}
