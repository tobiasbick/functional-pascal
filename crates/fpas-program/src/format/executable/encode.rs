//! Encode verified executable tables into tagged sections.

use fpas_bytecode::{Constant, Executable, ReturnConvention};

use super::super::debug;
use super::super::sections::{
    EncodedSection, TAGS, write_i64, write_u8, write_u16, write_u32, write_u64,
};
use super::super::{FormatError, check_limit, checked_u32};
use super::{
    CONSTANT_BOOLEAN, CONSTANT_FUNCTION, CONSTANT_INTEGER, CONSTANT_REAL, CONSTANT_STRING,
    CONSTANT_UNIT, FUNCTION_USES_SPAWN_TASKS, RETURN_UNIT, RETURN_VALUE, check_count,
};

pub(super) fn encode_strings(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_constants(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_sources(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_globals(executable: &Executable) -> Result<EncodedSection, FormatError> {
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
        write_u8(&mut bytes, u8::from(global.initializer.is_some()));
        if let Some(initializer) = global.initializer {
            write_u16(&mut bytes, initializer.function.get());
            write_u32(&mut bytes, initializer.instruction.get());
        }
    }
    Ok(EncodedSection {
        tag: TAGS[3],
        item_count: executable.globals.len(),
        bytes,
    })
}

pub(super) fn encode_records(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_enums(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_functions(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_instructions(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_source_runs(executable: &Executable) -> Result<EncodedSection, FormatError> {
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

pub(super) fn encode_entry(executable: &Executable) -> EncodedSection {
    let mut bytes = Vec::new();
    write_u16(&mut bytes, executable.entry.get());
    write_u16(&mut bytes, 0);
    EncodedSection {
        tag: TAGS[9],
        item_count: 1,
        bytes,
    }
}
