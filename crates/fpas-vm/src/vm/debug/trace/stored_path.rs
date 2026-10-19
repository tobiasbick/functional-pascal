//! Metadata-only selection of a bounded stored record path ending at a scalar.

use super::{model::DebugTracePathStep, selector::Part};
use fpas_bytecode::{DebugType, DebugTypeId, Executable};

pub(super) struct Fields {
    pub slots: Vec<usize>,
    pub steps: Vec<DebugTracePathStep>,
    pub indexed: bool,
    pub ty: DebugTypeId,
    pub type_name: String,
    pub suffix: String,
}

pub(super) fn bind<'a>(
    image: &'a Executable,
    mut ty: DebugTypeId,
    mut type_name: Option<&'a str>,
    parts: &[Part<'_>],
) -> Result<Fields, String> {
    if parts.len() > 16 {
        return Err("Trace stored paths allow at most 16 fields.".into());
    }
    let mut slots = Vec::new();
    let mut suffix = String::new();
    let mut steps = Vec::new();
    let mut indexed = false;
    for part in parts {
        let part = match part {
            Part::Field(name) => *name,
            Part::Index(index) => {
                let Some(DebugType::Array(element)) = image.debug_types.get(ty.get() as usize)
                else {
                    return Err("Trace index steps require an array.".into());
                };
                ty = *element;
                // Array element nominal names are not stored separately in current metadata.
                // Do not parse display strings as a substitute for a concrete debug type.
                type_name = None;
                steps.push(DebugTracePathStep::Index { index: *index });
                suffix.push_str(&format!("[{index}]"));
                indexed = true;
                continue;
            }
        };
        let Some(DebugType::Record(record)) = image.debug_types.get(ty.get() as usize) else {
            return Err("Trace field steps require ordinary stored records.".into());
        };
        let layout = image
            .records
            .get(record.get() as usize)
            .ok_or("Missing record metadata")?;
        let mut matches = layout.fields.iter().enumerate().filter(|(_, field)| {
            image
                .strings
                .get(field.name)
                .is_some_and(|name| name.eq_ignore_ascii_case(part))
        });
        let (slot, field) = matches.next().ok_or("Missing stored trace field")?;
        if matches.next().is_some() {
            return Err("Ambiguous stored trace field".into());
        }
        slots.push(slot);
        steps.push(DebugTracePathStep::Field { slot });
        suffix.push('.');
        suffix.push_str(image.strings.get(field.name).ok_or("Missing field name")?);
        ty = field.ty;
        type_name = field.type_name.and_then(|id| image.strings.get(id));
    }
    let scalar = super::binding::scalar_type(image, ty)
        .ok_or("Trace paths must end in a supported scalar")?;
    let type_name = type_name.unwrap_or(scalar);
    if type_name.len() > 4096 {
        return Err("Trace declared type metadata exceeds 4096 UTF-8 bytes.".into());
    }
    Ok(Fields {
        slots: if indexed { Vec::new() } else { slots },
        steps,
        indexed,
        ty,
        type_name: type_name.into(),
        suffix,
    })
}
