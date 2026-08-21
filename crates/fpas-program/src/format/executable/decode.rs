//! Decode tagged executable sections into verified tables.

use fpas_bytecode::{
    CodeRange, Constant, DebugTypeId, EnumLayout, EnumTypeId, EnumVariant, FunctionFlags,
    FunctionId, FunctionInfo, GlobalInfo, GlobalInitializer, Instruction, InstructionAddress,
    RecordField, RecordLayout, RecordProperty, ReturnConvention, SourceId, SourceRun, StringId,
    StringTable,
};

use super::super::debug::{self, DebugCounts};
use super::super::sections::{DecodedSection, SectionReader};
use super::super::{FormatError, check_limit};
use super::{
    CONSTANT_BOOLEAN, CONSTANT_FUNCTION, CONSTANT_INTEGER, CONSTANT_REAL, CONSTANT_STRING,
    CONSTANT_UNIT, FUNCTION_USES_SPAWN_TASKS, RETURN_UNIT, RETURN_VALUE, check_count, read_bool,
};

pub(super) fn decode_strings(
    section: DecodedSection<'_>,
    maximum: usize,
) -> Result<StringTable, FormatError> {
    check_count(section.tag, section.item_count, maximum)?;
    let mut reader = SectionReader::new(section.bytes, "string section");
    let mut cumulative = 0_usize;
    let mut values = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        let length = reader.u32("string_length")? as usize;
        cumulative = cumulative
            .checked_add(length)
            .ok_or(FormatError::LimitExceeded {
                field: "string_bytes",
                size: usize::MAX,
                maximum: fpas_bytecode::limits::MAX_STRING_BYTES,
            })?;
        check_limit(
            "string_bytes",
            cumulative,
            fpas_bytecode::limits::MAX_STRING_BYTES,
        )?;
        let bytes = reader.take(length, "string")?;
        let value = std::str::from_utf8(bytes).map_err(|_| FormatError::InvalidUtf8("string"))?;
        values.push(value.to_string());
    }
    reader.finish()?;
    Ok(StringTable::new(values))
}

pub(super) fn decode_constants(section: DecodedSection<'_>) -> Result<Vec<Constant>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_CONSTANTS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "constant section");
    let mut constants = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        let tag = reader.u8("constant_tag")?;
        let constant = match tag {
            CONSTANT_INTEGER => Constant::Integer(reader.i64("constant_integer")?),
            CONSTANT_REAL => Constant::Real(reader.u64("constant_real")?),
            CONSTANT_BOOLEAN => Constant::Boolean(read_bool(&mut reader, "constant_boolean")?),
            CONSTANT_STRING => Constant::String(StringId::new(reader.u32("constant_string")?)),
            CONSTANT_UNIT => Constant::Unit,
            CONSTANT_FUNCTION => Constant::Function {
                function: FunctionId::new(reader.u16("constant_function")?),
                task_bound: read_bool(&mut reader, "constant_task_bound")?,
            },
            value => {
                return Err(FormatError::InvalidValue {
                    field: "constant_tag",
                    value: u64::from(value),
                });
            }
        };
        constants.push(constant);
    }
    reader.finish()?;
    Ok(constants)
}

pub(super) fn decode_sources(section: DecodedSection<'_>) -> Result<Vec<StringId>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_SOURCE_PATHS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "source path section");
    let mut sources = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        sources.push(StringId::new(reader.u32("source_path_string")?));
    }
    reader.finish()?;
    Ok(sources)
}

pub(super) fn decode_globals(section: DecodedSection<'_>) -> Result<Vec<GlobalInfo>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_GLOBALS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "global section");
    let mut globals = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        globals.push(GlobalInfo {
            name: StringId::new(reader.u32("global_name")?),
            ty: DebugTypeId::new(reader.u32("global_type")?),
            mutable: read_bool(&mut reader, "global_mutable")?,
            initializer: if read_bool(&mut reader, "global_has_initializer")? {
                Some(GlobalInitializer {
                    function: FunctionId::new(reader.u16("global_initializer_function")?),
                    instruction: InstructionAddress::new(
                        reader.u32("global_initializer_instruction")?,
                    ),
                })
            } else {
                None
            },
        });
    }
    reader.finish()?;
    Ok(globals)
}

pub(super) fn decode_records(
    section: DecodedSection<'_>,
) -> Result<Vec<RecordLayout>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_RECORD_LAYOUTS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "record section");
    let mut records = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        let name = StringId::new(reader.u32("record_name")?);
        let count = reader.u32("record_field_count")? as usize;
        check_count(section.tag, count, fpas_bytecode::limits::MAX_LAYOUT_FIELDS)?;
        let mut fields = Vec::with_capacity(count);
        for _ in 0..count {
            fields.push(RecordField {
                name: StringId::new(reader.u32("record_field_name")?),
                ty: DebugTypeId::new(reader.u32("record_field_type")?),
            });
        }
        let property_count = reader.u32("record_property_count")? as usize;
        check_count(
            section.tag,
            property_count,
            fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
        )?;
        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            properties.push(RecordProperty {
                name: StringId::new(reader.u32("record_property_name")?),
                getter: StringId::new(reader.u32("record_property_getter")?),
            });
        }
        let method_count = reader.u32("record_method_count")? as usize;
        check_count(
            section.tag,
            method_count,
            fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
        )?;
        let mut methods = Vec::with_capacity(method_count);
        for _ in 0..method_count {
            methods.push(fpas_bytecode::RecordMethod {
                name: StringId::new(reader.u32("record_method_name")?),
                routine: StringId::new(reader.u32("record_method_routine")?),
            });
        }
        records.push(RecordLayout {
            name,
            fields,
            properties,
            methods,
        });
    }
    reader.finish()?;
    Ok(records)
}

pub(super) fn decode_enums(
    section: DecodedSection<'_>,
) -> Result<(Vec<EnumLayout>, Vec<EnumVariant>), FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_ENUM_LAYOUTS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "enum section");
    let variant_count = reader.u32("enum_variant_count")? as usize;
    check_count(
        section.tag,
        variant_count,
        fpas_bytecode::limits::MAX_ENUM_VARIANTS,
    )?;
    let mut enums = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        enums.push(EnumLayout {
            name: StringId::new(reader.u32("enum_name")?),
        });
    }
    let mut variants = Vec::with_capacity(variant_count);
    for _ in 0..variant_count {
        let owner = EnumTypeId::new(reader.u16("enum_variant_owner")?);
        let name = StringId::new(reader.u32("enum_variant_name")?);
        let field_count = reader.u32("enum_variant_field_count")? as usize;
        check_count(
            section.tag,
            field_count,
            fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
        )?;
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            fields.push(StringId::new(reader.u32("enum_variant_field_name")?));
        }
        let mut field_types = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            field_types.push(DebugTypeId::new(reader.u32("enum_variant_field_type")?));
        }
        variants.push(EnumVariant {
            owner,
            name,
            fields,
            field_types,
        });
    }
    reader.finish()?;
    Ok((enums, variants))
}

pub(super) fn decode_functions(
    section: DecodedSection<'_>,
) -> Result<Vec<FunctionInfo>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_FUNCTIONS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "function section");
    let mut functions = Vec::with_capacity(section.item_count);
    let mut debug_counts = DebugCounts::default();
    for _ in 0..section.item_count {
        let name = StringId::new(reader.u32("function_name")?);
        let start = InstructionAddress::new(reader.u32("function_code_start")?);
        let end = InstructionAddress::new(reader.u32("function_code_end")?);
        let arity = reader.u8("function_arity")?;
        let capture_count = reader.u16("function_capture_count")?;
        let register_count = reader.u16("function_register_count")?;
        let return_convention = match reader.u8("function_return_convention")? {
            RETURN_UNIT => ReturnConvention::Unit,
            RETURN_VALUE => ReturnConvention::Value,
            value => {
                return Err(FormatError::InvalidValue {
                    field: "function_return_convention",
                    value: u64::from(value),
                });
            }
        };
        let flags = reader.u16("function_flags")?;
        if flags & !FUNCTION_USES_SPAWN_TASKS != 0 {
            return Err(FormatError::UnsupportedFlags {
                field: "function",
                flags,
            });
        }
        functions.push(FunctionInfo {
            name,
            code: CodeRange::new(start, end),
            arity,
            capture_count,
            register_count,
            return_convention,
            flags: FunctionFlags {
                uses_spawn_tasks: flags & FUNCTION_USES_SPAWN_TASKS != 0,
            },
            debug: debug::decode(&mut reader, &mut debug_counts)?,
        });
    }
    reader.finish()?;
    Ok(functions)
}

pub(super) fn decode_instructions(
    section: DecodedSection<'_>,
) -> Result<Vec<Instruction>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_INSTRUCTIONS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "instruction section");
    let mut code = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        code.push(Instruction::from_word(reader.u64("instruction")?));
    }
    reader.finish()?;
    Ok(code)
}

pub(super) fn decode_source_runs(
    section: DecodedSection<'_>,
) -> Result<Vec<SourceRun>, FormatError> {
    check_count(
        section.tag,
        section.item_count,
        fpas_bytecode::limits::MAX_SOURCE_RUNS,
    )?;
    let mut reader = SectionReader::new(section.bytes, "source map section");
    let mut runs = Vec::with_capacity(section.item_count);
    for _ in 0..section.item_count {
        runs.push(SourceRun {
            instruction_start: InstructionAddress::new(reader.u32("source_instruction")?),
            source: SourceId::new(reader.u32("source_id")?),
            line: reader.u32("source_line")?,
            column: reader.u32("source_column")?,
        });
    }
    reader.finish()?;
    Ok(runs)
}

pub(super) fn decode_entry(section: DecodedSection<'_>) -> Result<FunctionId, FormatError> {
    if section.item_count != 1 {
        return Err(FormatError::SectionItemCount {
            tag: section.tag,
            count: section.item_count,
            maximum: 1,
        });
    }
    let mut reader = SectionReader::new(section.bytes, "entry section");
    let entry = FunctionId::new(reader.u16("entry_function")?);
    let flags = reader.u16("executable_flags")?;
    if flags != 0 {
        return Err(FormatError::UnsupportedFlags {
            field: "executable",
            flags,
        });
    }
    reader.finish()?;
    Ok(entry)
}
