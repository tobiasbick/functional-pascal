//! Binary conversion for portable debugger type graphs.

use fpas_bytecode::{DebugType, DebugTypeId, EnumTypeId, RecordTypeId};

use super::sections::{
    DecodedSection, EncodedSection, SectionReader, write_u8, write_u16, write_u32,
};
use super::{FormatError, check_limit, checked_u32};

const UNIT: u8 = 0;
const BOOLEAN: u8 = 1;
const INTEGER: u8 = 2;
const REAL: u8 = 3;
const STRING: u8 = 4;
const DYNAMIC: u8 = 5;
const ARRAY: u8 = 6;
const DICTIONARY: u8 = 7;
const RESULT: u8 = 8;
const OPTION: u8 = 9;
const FUNCTION: u8 = 10;
const RECORD: u8 = 11;
const ENUM: u8 = 12;
const CELL: u8 = 13;
const TASK: u8 = 14;
const CHANNEL: u8 = 15;

pub(super) fn encode(types: &[DebugType], tag: u16) -> Result<EncodedSection, FormatError> {
    check_limit(
        "debug_types",
        types.len(),
        fpas_bytecode::limits::MAX_DEBUG_TYPES,
    )?;
    let mut bytes = Vec::new();
    for ty in types {
        match ty {
            DebugType::Unit => write_u8(&mut bytes, UNIT),
            DebugType::Boolean => write_u8(&mut bytes, BOOLEAN),
            DebugType::Integer => write_u8(&mut bytes, INTEGER),
            DebugType::Real => write_u8(&mut bytes, REAL),
            DebugType::String => write_u8(&mut bytes, STRING),
            DebugType::Dynamic => write_u8(&mut bytes, DYNAMIC),
            DebugType::Array(inner) => {
                write_u8(&mut bytes, ARRAY);
                write_u32(&mut bytes, inner.get());
            }
            DebugType::Dictionary { key, value } => {
                write_u8(&mut bytes, DICTIONARY);
                write_u32(&mut bytes, key.get());
                write_u32(&mut bytes, value.get());
            }
            DebugType::Result { ok, error } => {
                write_u8(&mut bytes, RESULT);
                write_u32(&mut bytes, ok.get());
                write_u32(&mut bytes, error.get());
            }
            DebugType::Option(inner) => {
                write_u8(&mut bytes, OPTION);
                write_u32(&mut bytes, inner.get());
            }
            DebugType::Function { parameters, result } => {
                write_u8(&mut bytes, FUNCTION);
                write_u32(
                    &mut bytes,
                    checked_u32("debug_function_parameters", parameters.len())?,
                );
                for parameter in parameters {
                    write_u32(&mut bytes, parameter.get());
                }
                write_u32(&mut bytes, result.get());
            }
            DebugType::Record(layout) => {
                write_u8(&mut bytes, RECORD);
                write_u16(&mut bytes, layout.get());
            }
            DebugType::Enum(layout) => {
                write_u8(&mut bytes, ENUM);
                write_u16(&mut bytes, layout.get());
            }
            DebugType::Cell(inner) => {
                write_u8(&mut bytes, CELL);
                write_u32(&mut bytes, inner.get());
            }
            DebugType::Task(inner) => {
                write_u8(&mut bytes, TASK);
                write_u32(&mut bytes, inner.get());
            }
            DebugType::Channel(inner) => {
                write_u8(&mut bytes, CHANNEL);
                write_u32(&mut bytes, inner.get());
            }
        }
    }
    Ok(EncodedSection {
        tag,
        item_count: types.len(),
        bytes,
    })
}

pub(super) fn decode(section: DecodedSection<'_>) -> Result<Vec<DebugType>, FormatError> {
    check_limit(
        "debug_types",
        section.item_count,
        fpas_bytecode::limits::MAX_DEBUG_TYPES,
    )?;
    let mut reader = SectionReader::new(section.bytes, "debug type section");
    let mut types = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        let ty = match reader.u8("debug_type_tag")? {
            UNIT => DebugType::Unit,
            BOOLEAN => DebugType::Boolean,
            INTEGER => DebugType::Integer,
            REAL => DebugType::Real,
            STRING => DebugType::String,
            DYNAMIC => DebugType::Dynamic,
            ARRAY => DebugType::Array(read_id(&mut reader, "array_element_type")?),
            DICTIONARY => DebugType::Dictionary {
                key: read_id(&mut reader, "dictionary_key_type")?,
                value: read_id(&mut reader, "dictionary_value_type")?,
            },
            RESULT => DebugType::Result {
                ok: read_id(&mut reader, "result_ok_type")?,
                error: read_id(&mut reader, "result_error_type")?,
            },
            OPTION => DebugType::Option(read_id(&mut reader, "option_inner_type")?),
            FUNCTION => {
                let count = reader.u32("debug_function_parameter_count")? as usize;
                check_limit(
                    "debug_function_parameters",
                    count,
                    fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
                )?;
                let mut parameters = Vec::with_capacity(count);
                for _ in 0..count {
                    parameters.push(read_id(&mut reader, "debug_function_parameter_type")?);
                }
                DebugType::Function {
                    parameters,
                    result: read_id(&mut reader, "debug_function_result_type")?,
                }
            }
            RECORD => DebugType::Record(RecordTypeId::new(reader.u16("debug_record_type")?)),
            ENUM => DebugType::Enum(EnumTypeId::new(reader.u16("debug_enum_type")?)),
            CELL => DebugType::Cell(read_id(&mut reader, "cell_inner_type")?),
            TASK => DebugType::Task(read_id(&mut reader, "task_result_type")?),
            CHANNEL => DebugType::Channel(read_id(&mut reader, "channel_element_type")?),
            value => {
                return Err(FormatError::InvalidValue {
                    field: "debug_type_tag",
                    value: u64::from(value),
                });
            }
        };
        types.push(ty);
    }
    reader.finish()?;
    Ok(types)
}

fn read_id(
    reader: &mut SectionReader<'_>,
    field: &'static str,
) -> Result<DebugTypeId, FormatError> {
    Ok(DebugTypeId::new(reader.u32(field)?))
}
