//! Admit native host-information layouts against the shared field contract.
use super::super::{VmError, diagnostics};
use crate::{DebugType, DebugTypeId, Executable, InstructionAddress};
use fpas_builtins::host_info::{self, Shape};

pub(super) fn validate(image: &Executable, address: InstructionAddress) -> Result<(), VmError> {
    for layout in &image.records {
        let Some(name) = image.strings.get(layout.name) else {
            continue;
        };
        let Some(expected) = host_info::record(name) else {
            continue;
        };
        if layout.fields.len() != expected.fields.len()
            || !layout
                .fields
                .iter()
                .zip(expected.fields)
                .all(|(actual, (name, shape))| {
                    image.strings.get(actual.name) == Some(*name)
                        && matches_shape(image, actual.ty, *shape)
                })
        {
            return Err(diagnostics::internal(
                image,
                address,
                format!("Native host-information record contract does not match {name}"),
            ));
        }
    }
    Ok(())
}
fn matches_shape(image: &Executable, id: DebugTypeId, shape: Shape) -> bool {
    let Some(actual) = image.debug_types.get(id.get() as usize) else {
        return false;
    };
    match (actual, shape) {
        (DebugType::Boolean, Shape::Boolean)
        | (DebugType::Integer, Shape::Integer)
        | (DebugType::Real, Shape::Real)
        | (DebugType::String, Shape::String) => true,
        (DebugType::Option(id), Shape::Option(inner))
        | (DebugType::Array(id), Shape::Array(inner)) => matches_shape(image, *id, *inner),
        (DebugType::Result { ok, error }, Shape::Result(inner)) => {
            matches_shape(image, *ok, *inner)
                && matches!(image.debug_types.get(error.get() as usize), Some(DebugType::Enum(id)) if
                image.enums.get(id.get() as usize).and_then(|e| image.strings.get(e.name)) == Some("Std.Os.System.SystemError"))
        }
        (DebugType::Record(id), Shape::Record(name)) => image
            .records
            .get(id.get() as usize)
            .and_then(|r| image.strings.get(r.name))
            .is_some_and(|actual| actual.eq_ignore_ascii_case(name)),
        _ => false,
    }
}
