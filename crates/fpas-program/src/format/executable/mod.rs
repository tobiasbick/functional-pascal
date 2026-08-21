//! Explicit little-endian register executable section conversion.

mod decode;
mod encode;

use fpas_bytecode::{Executable, SourceMap, VerifiedExecutable};

use super::debug_types;
use super::sections::{DecodedSection, SectionReader, TAGS};
use super::{FormatError, sections};

pub(super) const CONSTANT_INTEGER: u8 = 0;
pub(super) const CONSTANT_REAL: u8 = 1;
pub(super) const CONSTANT_BOOLEAN: u8 = 2;
pub(super) const CONSTANT_STRING: u8 = 3;
pub(super) const CONSTANT_UNIT: u8 = 4;
pub(super) const CONSTANT_FUNCTION: u8 = 5;
pub(super) const RETURN_UNIT: u8 = 0;
pub(super) const RETURN_VALUE: u8 = 1;
pub(super) const FUNCTION_USES_SPAWN_TASKS: u16 = 1;

pub(super) fn encode(executable: &Executable) -> Result<Vec<u8>, FormatError> {
    let sections = vec![
        encode::encode_strings(executable)?,
        encode::encode_constants(executable)?,
        encode::encode_sources(executable)?,
        encode::encode_globals(executable)?,
        encode::encode_records(executable)?,
        encode::encode_enums(executable)?,
        encode::encode_functions(executable)?,
        encode::encode_instructions(executable)?,
        encode::encode_source_runs(executable)?,
        encode::encode_entry(executable),
        debug_types::encode(&executable.debug_types, TAGS[10])?,
    ];
    sections::encode(sections)
}

pub(super) fn decode(payload: &[u8]) -> Result<VerifiedExecutable, FormatError> {
    let sections = sections::decode(payload)?;
    let strings =
        decode::decode_strings(section(&sections, 0), fpas_bytecode::limits::MAX_STRINGS)?;
    let constants = decode::decode_constants(section(&sections, 1))?;
    let sources = decode::decode_sources(section(&sections, 2))?;
    let globals = decode::decode_globals(section(&sections, 3))?;
    let records = decode::decode_records(section(&sections, 4))?;
    let (enums, enum_variants) = decode::decode_enums(section(&sections, 5))?;
    let functions = decode::decode_functions(section(&sections, 6))?;
    let code = decode::decode_instructions(section(&sections, 7))?;
    let runs = decode::decode_source_runs(section(&sections, 8))?;
    let entry = decode::decode_entry(section(&sections, 9))?;
    let debug_types = debug_types::decode(section(&sections, 10))?;
    Executable {
        code,
        functions,
        constants,
        strings,
        globals,
        records,
        enums,
        enum_variants,
        debug_types,
        source_map: SourceMap { sources, runs },
        entry,
    }
    .verify()
    .map_err(|error| FormatError::Image(crate::ImageError::Executable(error)))
}

pub(super) fn section<'a>(sections: &'a [DecodedSection<'a>], index: usize) -> DecodedSection<'a> {
    sections[index]
}

pub(super) fn read_bool(
    reader: &mut SectionReader<'_>,
    field: &'static str,
) -> Result<bool, FormatError> {
    match reader.u8(field)? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(FormatError::InvalidValue {
            field,
            value: u64::from(value),
        }),
    }
}

pub(super) fn check_count(tag: u16, count: usize, maximum: usize) -> Result<(), FormatError> {
    if count > maximum {
        return Err(FormatError::SectionItemCount {
            tag,
            count,
            maximum,
        });
    }
    Ok(())
}
