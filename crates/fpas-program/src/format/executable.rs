//! Explicit little-endian register executable section conversion.

use fpas_bytecode::{
    CodeRange, Constant, DebugTypeId, EnumLayout, EnumTypeId, EnumVariant, Executable,
    FunctionFlags, FunctionId, FunctionInfo, GlobalInfo, Instruction, InstructionAddress,
    RecordField, RecordLayout, RecordProperty, ReturnConvention, SourceId, SourceMap, SourceRun,
    StringId, StringTable, VerifiedExecutable,
};

use super::debug::{self, DebugCounts};
use super::debug_types;
use super::sections::{
    DecodedSection, EncodedSection, SectionReader, TAGS, write_i64, write_u8, write_u16, write_u32,
    write_u64,
};
use super::{FormatError, check_limit, checked_u32, sections};

const CONSTANT_INTEGER: u8 = 0;
const CONSTANT_REAL: u8 = 1;
const CONSTANT_BOOLEAN: u8 = 2;
const CONSTANT_STRING: u8 = 3;
const CONSTANT_UNIT: u8 = 4;
const CONSTANT_FUNCTION: u8 = 5;
const RETURN_UNIT: u8 = 0;
const RETURN_VALUE: u8 = 1;
const FUNCTION_USES_SPAWN_TASKS: u16 = 1;

pub(super) fn encode(executable: &Executable) -> Result<Vec<u8>, FormatError> {
    let sections = vec![
        encode_strings(executable)?,
        encode_constants(executable)?,
        encode_sources(executable)?,
        encode_globals(executable)?,
        encode_records(executable)?,
        encode_enums(executable)?,
        encode_functions(executable)?,
        encode_instructions(executable)?,
        encode_source_runs(executable)?,
        encode_entry(executable),
        debug_types::encode(&executable.debug_types, TAGS[10])?,
    ];
    sections::encode(sections)
}

pub(super) fn decode(payload: &[u8]) -> Result<VerifiedExecutable, FormatError> {
    let sections = sections::decode(payload)?;
    let strings = decode_strings(section(&sections, 0), fpas_bytecode::limits::MAX_STRINGS)?;
    let constants = decode_constants(section(&sections, 1))?;
    let sources = decode_sources(section(&sections, 2))?;
    let globals = decode_globals(section(&sections, 3))?;
    let records = decode_records(section(&sections, 4))?;
    let (enums, enum_variants) = decode_enums(section(&sections, 5))?;
    let functions = decode_functions(section(&sections, 6))?;
    let code = decode_instructions(section(&sections, 7))?;
    let runs = decode_source_runs(section(&sections, 8))?;
    let entry = decode_entry(section(&sections, 9))?;
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

fn section<'a>(sections: &'a [DecodedSection<'a>], index: usize) -> DecodedSection<'a> {
    sections[index]
}

fn encode_strings(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[0],
        executable.strings.len(),
        fpas_bytecode::limits::MAX_STRINGS,
    )?;
    let mut bytes = Vec::new();
    let mut cumulative = 0_usize;
    for value in executable.strings.iter() {
        cumulative = cumulative
            .checked_add(value.len())
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
        write_u32(&mut bytes, checked_u32("string_length", value.len())?);
        bytes.extend_from_slice(value.as_bytes());
    }
    Ok(EncodedSection {
        tag: TAGS[0],
        item_count: executable.strings.len(),
        bytes,
    })
}

fn decode_strings(section: DecodedSection<'_>, maximum: usize) -> Result<StringTable, FormatError> {
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

fn encode_constants(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[1],
        executable.constants.len(),
        fpas_bytecode::limits::MAX_CONSTANTS,
    )?;
    let mut bytes = Vec::new();
    for constant in &executable.constants {
        match constant {
            Constant::Integer(value) => {
                write_u8(&mut bytes, CONSTANT_INTEGER);
                write_i64(&mut bytes, *value);
            }
            Constant::Real(bits) => {
                write_u8(&mut bytes, CONSTANT_REAL);
                write_u64(&mut bytes, *bits);
            }
            Constant::Boolean(value) => {
                write_u8(&mut bytes, CONSTANT_BOOLEAN);
                write_u8(&mut bytes, u8::from(*value));
            }
            Constant::String(value) => {
                write_u8(&mut bytes, CONSTANT_STRING);
                write_u32(&mut bytes, value.get());
            }
            Constant::Unit => write_u8(&mut bytes, CONSTANT_UNIT),
            Constant::Function {
                function,
                task_bound,
            } => {
                write_u8(&mut bytes, CONSTANT_FUNCTION);
                write_u16(&mut bytes, function.get());
                write_u8(&mut bytes, u8::from(*task_bound));
            }
        }
    }
    Ok(EncodedSection {
        tag: TAGS[1],
        item_count: executable.constants.len(),
        bytes,
    })
}

fn decode_constants(section: DecodedSection<'_>) -> Result<Vec<Constant>, FormatError> {
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

fn encode_sources(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[2],
        executable.source_map.sources.len(),
        fpas_bytecode::limits::MAX_SOURCE_PATHS,
    )?;
    let mut bytes = Vec::with_capacity(executable.source_map.sources.len() * 4);
    for source in &executable.source_map.sources {
        write_u32(&mut bytes, source.get());
    }
    Ok(EncodedSection {
        tag: TAGS[2],
        item_count: executable.source_map.sources.len(),
        bytes,
    })
}

fn decode_sources(section: DecodedSection<'_>) -> Result<Vec<StringId>, FormatError> {
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

fn encode_globals(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[3],
        executable.globals.len(),
        fpas_bytecode::limits::MAX_GLOBALS,
    )?;
    let mut bytes = Vec::new();
    for global in &executable.globals {
        write_u32(&mut bytes, global.name.get());
        write_u32(&mut bytes, global.ty.get());
        write_u8(&mut bytes, u8::from(global.mutable));
    }
    Ok(EncodedSection {
        tag: TAGS[3],
        item_count: executable.globals.len(),
        bytes,
    })
}

fn decode_globals(section: DecodedSection<'_>) -> Result<Vec<GlobalInfo>, FormatError> {
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
        });
    }
    reader.finish()?;
    Ok(globals)
}

fn encode_records(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[4],
        executable.records.len(),
        fpas_bytecode::limits::MAX_RECORD_LAYOUTS,
    )?;
    let mut bytes = Vec::new();
    for record in &executable.records {
        check_count(
            TAGS[4],
            record.fields.len(),
            fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
        )?;
        write_u32(&mut bytes, record.name.get());
        write_u32(
            &mut bytes,
            checked_u32("record_fields", record.fields.len())?,
        );
        for field in &record.fields {
            write_u32(&mut bytes, field.name.get());
            write_u32(&mut bytes, field.ty.get());
        }
        write_u32(
            &mut bytes,
            checked_u32("record_properties", record.properties.len())?,
        );
        for property in &record.properties {
            write_u32(&mut bytes, property.name.get());
            write_u32(&mut bytes, property.getter.get());
        }
        write_u32(
            &mut bytes,
            checked_u32("record_methods", record.methods.len())?,
        );
        for method in &record.methods {
            write_u32(&mut bytes, method.name.get());
            write_u32(&mut bytes, method.routine.get());
        }
    }
    Ok(EncodedSection {
        tag: TAGS[4],
        item_count: executable.records.len(),
        bytes,
    })
}

fn decode_records(section: DecodedSection<'_>) -> Result<Vec<RecordLayout>, FormatError> {
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

fn encode_enums(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[5],
        executable.enums.len(),
        fpas_bytecode::limits::MAX_ENUM_LAYOUTS,
    )?;
    check_count(
        TAGS[5],
        executable.enum_variants.len(),
        fpas_bytecode::limits::MAX_ENUM_VARIANTS,
    )?;
    let mut bytes = Vec::new();
    write_u32(
        &mut bytes,
        checked_u32("enum_variant_count", executable.enum_variants.len())?,
    );
    for enumeration in &executable.enums {
        write_u32(&mut bytes, enumeration.name.get());
    }
    for variant in &executable.enum_variants {
        check_count(
            TAGS[5],
            variant.fields.len(),
            fpas_bytecode::limits::MAX_LAYOUT_FIELDS,
        )?;
        write_u16(&mut bytes, variant.owner.get());
        write_u32(&mut bytes, variant.name.get());
        write_u32(
            &mut bytes,
            checked_u32("enum_variant_fields", variant.fields.len())?,
        );
        for field in &variant.fields {
            write_u32(&mut bytes, field.get());
        }
        for ty in &variant.field_types {
            write_u32(&mut bytes, ty.get());
        }
    }
    Ok(EncodedSection {
        tag: TAGS[5],
        item_count: executable.enums.len(),
        bytes,
    })
}

fn decode_enums(
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

fn encode_functions(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[6],
        executable.functions.len(),
        fpas_bytecode::limits::MAX_FUNCTIONS,
    )?;
    let mut bytes = Vec::new();
    for function in &executable.functions {
        write_u32(&mut bytes, function.name.get());
        write_u32(&mut bytes, function.code.start.get());
        write_u32(&mut bytes, function.code.end.get());
        write_u8(&mut bytes, function.arity);
        write_u16(&mut bytes, function.capture_count);
        write_u16(&mut bytes, function.register_count);
        write_u8(
            &mut bytes,
            match function.return_convention {
                ReturnConvention::Unit => RETURN_UNIT,
                ReturnConvention::Value => RETURN_VALUE,
            },
        );
        let flags = if function.flags.uses_spawn_tasks {
            FUNCTION_USES_SPAWN_TASKS
        } else {
            0
        };
        write_u16(&mut bytes, flags);
        debug::encode(&mut bytes, &function.debug)?;
    }
    Ok(EncodedSection {
        tag: TAGS[6],
        item_count: executable.functions.len(),
        bytes,
    })
}

fn decode_functions(section: DecodedSection<'_>) -> Result<Vec<FunctionInfo>, FormatError> {
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

fn encode_instructions(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[7],
        executable.code.len(),
        fpas_bytecode::limits::MAX_INSTRUCTIONS,
    )?;
    let mut bytes = Vec::with_capacity(executable.code.len() * 8);
    for instruction in &executable.code {
        write_u64(&mut bytes, instruction.word());
    }
    Ok(EncodedSection {
        tag: TAGS[7],
        item_count: executable.code.len(),
        bytes,
    })
}

fn decode_instructions(section: DecodedSection<'_>) -> Result<Vec<Instruction>, FormatError> {
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

fn encode_source_runs(executable: &Executable) -> Result<EncodedSection, FormatError> {
    check_count(
        TAGS[8],
        executable.source_map.runs.len(),
        fpas_bytecode::limits::MAX_SOURCE_RUNS,
    )?;
    let mut bytes = Vec::with_capacity(executable.source_map.runs.len() * 16);
    for run in &executable.source_map.runs {
        write_u32(&mut bytes, run.instruction_start.get());
        write_u32(&mut bytes, run.source.get());
        write_u32(&mut bytes, run.line);
        write_u32(&mut bytes, run.column);
    }
    Ok(EncodedSection {
        tag: TAGS[8],
        item_count: executable.source_map.runs.len(),
        bytes,
    })
}

fn decode_source_runs(section: DecodedSection<'_>) -> Result<Vec<SourceRun>, FormatError> {
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

fn encode_entry(executable: &Executable) -> EncodedSection {
    let mut bytes = Vec::new();
    write_u16(&mut bytes, executable.entry.get());
    write_u16(&mut bytes, 0);
    EncodedSection {
        tag: TAGS[9],
        item_count: 1,
        bytes,
    }
}

fn decode_entry(section: DecodedSection<'_>) -> Result<FunctionId, FormatError> {
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

fn read_bool(reader: &mut SectionReader<'_>, field: &'static str) -> Result<bool, FormatError> {
    match reader.u8(field)? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(FormatError::InvalidValue {
            field,
            value: u64::from(value),
        }),
    }
}

fn check_count(tag: u16, count: usize, maximum: usize) -> Result<(), FormatError> {
    if count > maximum {
        return Err(FormatError::SectionItemCount {
            tag,
            count,
            maximum,
        });
    }
    Ok(())
}
