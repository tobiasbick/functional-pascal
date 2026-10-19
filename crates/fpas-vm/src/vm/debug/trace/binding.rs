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
        let parts = selector.split('.').collect::<Vec<_>>();
        if parts.iter().any(|part| !identifier(part)) {
            return Err(format!(
                "Trace `{selector}` requires a dotted identifier path; calls and indexes are unsupported."
            ));
        }
        let mut candidate = None;
        for (index, global) in image.globals.iter().enumerate() {
            let Some(name) = image.strings.get(global.name) else {
                continue;
            };
            if name.len() > selector.len() {
                continue;
            }
            let root_parts = name.split('.').collect::<Vec<_>>();
            if !global.mutable
                || root_parts.len() > parts.len()
                || !root_parts
                    .iter()
                    .zip(&parts)
                    .all(|(left, right)| left.eq_ignore_ascii_case(right))
            {
                continue;
            }
            let tail = &parts[root_parts.len()..];
            if tail.len() > 16 {
                continue;
            }
            let mut ty = global.ty;
            let mut type_name = global.type_name.and_then(|id| image.strings.get(id));
            let mut slots = Vec::new();
            let mut canonical = name.to_string();
            for part in tail {
                let Some(DebugType::Record(record)) = image.debug_types.get(ty.get() as usize)
                else {
                    break;
                };
                let Some(layout) = image.records.get(record.get() as usize) else {
                    break;
                };
                let mut matching = layout.fields.iter().enumerate().filter(|(_, field)| {
                    image
                        .strings
                        .get(field.name)
                        .is_some_and(|name| name.eq_ignore_ascii_case(part))
                });
                let Some((slot, field)) = matching.next() else {
                    break;
                };
                if matching.next().is_some() {
                    break;
                }
                slots.push(slot);
                canonical.push('.');
                canonical.push_str(image.strings.get(field.name).unwrap_or(part));
                ty = field.ty;
                type_name = field.type_name.and_then(|id| image.strings.get(id));
            }
            if slots.len() != tail.len() {
                continue;
            }
            let Some(scalar_name) = scalar_type(image, ty) else {
                continue;
            };
            let type_name = type_name.unwrap_or(scalar_name);
            if type_name.len() > 4096 {
                return Err("Trace declared type metadata exceeds 4096 UTF-8 bytes.".into());
            }
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
                    field_slots: slots,
                },
                type_name: type_name.into(),
                ty,
            });
        }
        let Some(watch) = candidate else {
            return Err(format!(
                "Trace `{selector}` requires one mutable executable global or stored scalar record field, using the full metadata unit name. Use Routine::Name for scalar locals/parameters; captures, aggregates and aliases are unsupported."
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

pub(super) fn identifier(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
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
