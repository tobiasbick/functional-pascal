//! Bounded essential error identities and concrete field/Cause metadata.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::super::sections::{SectionReader, write_u8, write_u16, write_u32};
use super::super::{FormatError, check_limit, checked_u32};
use fpas_bytecode::{
    EnumTypeId, ErrorCause, ErrorInstanceKey, ErrorVariant, StringId, TypeIdentity,
};

pub(super) fn encode_key(
    bytes: &mut Vec<u8>,
    key: Option<&ErrorInstanceKey>,
) -> Result<(), FormatError> {
    write_u8(bytes, u8::from(key.is_some()));
    if let Some(key) = key {
        write_json(
            bytes,
            &serde_json::to_vec(key).map_err(|_| invalid("error_instance_key"))?,
        )?;
    }
    Ok(())
}

pub(super) fn decode_key(
    reader: &mut SectionReader<'_>,
) -> Result<Option<ErrorInstanceKey>, FormatError> {
    if !super::read_bool(reader, "error_family_kind")? {
        return Ok(None);
    }
    let key: ErrorInstanceKey =
        serde_json::from_slice(read_json(reader)?).map_err(|_| invalid("error_instance_key"))?;
    key.validate(0).map_err(|_| invalid("error_instance_key"))?;
    Ok(Some(key))
}

pub(super) fn encode_variant(
    bytes: &mut Vec<u8>,
    error: Option<&ErrorVariant>,
) -> Result<(), FormatError> {
    write_u8(bytes, u8::from(error.is_some()));
    if let Some(error) = error {
        write_u32(
            bytes,
            error.message.map_or(u32::MAX, |message| message.get()),
        );
        write_json(
            bytes,
            &serde_json::to_vec(&error.field_types).map_err(|_| invalid("error_field_types"))?,
        )?;
        write_u8(bytes, u8::from(error.cause.is_some()));
        if let Some(cause) = error.cause {
            write_u16(bytes, cause.field);
            write_u16(bytes, cause.family.get());
        }
    }
    Ok(())
}

pub(super) fn decode_variant(
    reader: &mut SectionReader<'_>,
) -> Result<Option<ErrorVariant>, FormatError> {
    if !super::read_bool(reader, "error_variant_kind")? {
        return Ok(None);
    }
    let message = match reader.u32("error_message")? {
        u32::MAX => None,
        value => Some(StringId::new(value)),
    };
    let mut decoder = serde_json::Deserializer::from_slice(read_json(reader)?);
    let field_types =
        TypeIdentity::deserialize_list(&mut decoder).map_err(|_| invalid("error_field_types"))?;
    decoder.end().map_err(|_| invalid("error_field_types"))?;
    TypeIdentity::validate_all(&field_types, 0).map_err(|_| invalid("error_field_types"))?;
    let cause = if super::read_bool(reader, "error_cause_present")? {
        Some(ErrorCause {
            field: reader.u16("error_cause_field")?,
            family: EnumTypeId::new(reader.u16("error_cause_family")?),
        })
    } else {
        None
    };
    Ok(Some(ErrorVariant {
        field_types,
        message,
        cause,
    }))
}

fn write_json(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), FormatError> {
    check_limit(
        "error_metadata_bytes",
        value.len(),
        fpas_bytecode::limits::MAX_STRING_BYTES,
    )?;
    write_u32(bytes, checked_u32("error_metadata_bytes", value.len())?);
    bytes.extend_from_slice(value);
    Ok(())
}

fn read_json<'a>(reader: &mut SectionReader<'a>) -> Result<&'a [u8], FormatError> {
    let length = reader.u32("error_metadata_bytes")? as usize;
    check_limit(
        "error_metadata_bytes",
        length,
        fpas_bytecode::limits::MAX_STRING_BYTES,
    )?;
    reader.take(length, "error_metadata")
}

fn invalid(field: &'static str) -> FormatError {
    FormatError::InvalidValue { field, value: 0 }
}
