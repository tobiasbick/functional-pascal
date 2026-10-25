//! Verify shared native record shapes before native value construction.

use fpas_bytecode::{DebugType, DebugTypeId, Executable, InstructionAddress};
use fpas_std::{native_errors::ErrorFieldType, native_records};

use super::super::{VmError, diagnostics};

pub(super) fn require(
    image: &Executable,
    name: &str,
    address: InstructionAddress,
) -> Result<(), VmError> {
    let failure = || {
        diagnostics::internal(
            image,
            address,
            format!("Native record image contract does not match {name}"),
        )
    };
    let expected = native_records::find(name).ok_or_else(failure)?;
    let mut layouts = image.records.iter().filter(|record| {
        image
            .strings
            .get(record.name)
            .is_some_and(|value| value.eq_ignore_ascii_case(name))
    });
    let layout = layouts.next().ok_or_else(failure)?;
    if layouts.next().is_some() || layout.fields.len() != expected.fields.len() {
        return Err(failure());
    }
    for (field, (name, shape)) in layout.fields.iter().zip(expected.fields) {
        if image.strings.get(field.name) != Some(*name) || !matches_shape(image, field.ty, *shape) {
            return Err(failure());
        }
        if let ErrorFieldType::Record(name) | ErrorFieldType::OptionalRecord(name) = shape {
            require(image, name, address)?;
        }
    }
    Ok(())
}

fn matches_shape(image: &Executable, id: DebugTypeId, expected: ErrorFieldType) -> bool {
    let Some(actual) = image.debug_types.get(id.get() as usize) else {
        return false;
    };
    match (actual, expected) {
        (DebugType::String, ErrorFieldType::String)
        | (DebugType::Integer, ErrorFieldType::Integer) => true,
        (DebugType::Array(id), ErrorFieldType::Strings) => {
            matches_shape(image, *id, ErrorFieldType::String)
        }
        (DebugType::Option(id), ErrorFieldType::OptionalString) => {
            matches_shape(image, *id, ErrorFieldType::String)
        }
        (DebugType::Option(id), ErrorFieldType::OptionalInteger) => {
            matches_shape(image, *id, ErrorFieldType::Integer)
        }
        (DebugType::Option(id), ErrorFieldType::OptionalRecord(name)) => {
            matches_shape(image, *id, ErrorFieldType::Record(name))
        }
        (DebugType::Record(id), ErrorFieldType::Record(name)) => image
            .records
            .get(usize::from(id.get()))
            .and_then(|record| image.strings.get(record.name))
            .is_some_and(|value| value.eq_ignore_ascii_case(name)),
        (DebugType::Enum(id), ErrorFieldType::Host | ErrorFieldType::Net) => {
            expected.cause_family().is_some_and(|family| {
                image
                    .enums
                    .get(usize::from(id.get()))
                    .and_then(|owner| image.strings.get(owner.name))
                    .is_some_and(|name| name.eq_ignore_ascii_case(family.name()))
            })
        }
        _ => false,
    }
}
