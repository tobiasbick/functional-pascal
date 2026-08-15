//! Metadata resource limits, table references, layouts, and sparse source maps.

use std::collections::HashMap;

use crate::{
    EnumVariantId, FunctionId, FunctionInfo, InstructionAddress, Opcode, RecordTypeId, StringId,
    limits,
};

use super::calls::validate_window;
use super::{ValidationError, ValidationErrorKind};

pub(super) fn validate_tables(executable: &crate::Executable) -> Result<(), ValidationError> {
    let mut strings = HashMap::<&str, StringId>::new();
    for (index, value) in executable.strings.iter().enumerate() {
        let id = StringId::try_from_index(index).map_err(|_| {
            ValidationError::executable(ValidationErrorKind::ResourceLimit {
                resource: "strings",
                actual: executable.strings.len(),
                maximum: limits::MAX_STRINGS,
            })
        })?;
        if let Some(first) = strings.insert(value, id) {
            return Err(ValidationError::executable(
                ValidationErrorKind::DuplicateString {
                    duplicate: id,
                    first,
                },
            ));
        }
    }

    for constant in &executable.constants {
        match constant {
            crate::Constant::String(string) => validate_string(executable, *string, "constant")?,
            crate::Constant::Function {
                function,
                task_bound,
            } => {
                let function_info =
                    validate_function_reference(executable, *function, "constant function")?;
                if function_info.capture_count != 0 || *task_bound {
                    return Err(ValidationError::executable(
                        ValidationErrorKind::ConstantFunction {
                            function: function.get(),
                            captures: function_info.capture_count,
                            task_bound: *task_bound,
                        },
                    ));
                }
            }
            crate::Constant::Integer(_)
            | crate::Constant::Real(_)
            | crate::Constant::Boolean(_)
            | crate::Constant::Unit => {}
        }
    }
    for global in &executable.globals {
        validate_string(executable, global.name, "global name")?;
    }
    for function in &executable.functions {
        validate_string(executable, function.name, "function name")?;
        for binding in &function.debug.bindings {
            validate_string(executable, binding.name, "debug binding name")?;
            validate_string(executable, binding.type_name, "debug binding type")?;
        }
    }
    for record in &executable.records {
        validate_string(executable, record.name, "record name")?;
        for field in &record.fields {
            validate_string(executable, field.name, "record field name")?;
        }
        for property in &record.properties {
            validate_string(executable, property.name, "record property name")?;
            validate_string(executable, property.getter, "record property getter")?;
        }
        for method in &record.methods {
            validate_string(executable, method.name, "record method name")?;
            validate_string(executable, method.routine, "record method routine")?;
        }
    }
    for enumeration in &executable.enums {
        validate_string(executable, enumeration.name, "enum name")?;
    }
    for variant in &executable.enum_variants {
        if usize::from(variant.owner.get()) >= executable.enums.len() {
            return Err(ValidationError::executable(
                ValidationErrorKind::TableReference {
                    table: "enum layouts",
                    operand: "variant owner",
                    actual: u64::from(variant.owner.get()),
                    length: executable.enums.len(),
                },
            ));
        }
        validate_string(executable, variant.name, "enum variant name")?;
        for field in &variant.fields {
            validate_string(executable, *field, "enum field name")?;
        }
    }
    for source in &executable.source_map.sources {
        validate_string(executable, *source, "source path")?;
    }
    Ok(())
}

fn validate_string(
    executable: &crate::Executable,
    id: StringId,
    owner: &'static str,
) -> Result<(), ValidationError> {
    if executable.strings.get(id).is_some() {
        Ok(())
    } else {
        Err(ValidationError::executable(
            ValidationErrorKind::StringReference {
                owner,
                actual: id.get(),
                strings: executable.strings.len(),
            },
        ))
    }
}

fn validate_function_reference<'a>(
    executable: &'a crate::Executable,
    id: FunctionId,
    operand: &'static str,
) -> Result<&'a crate::FunctionInfo, ValidationError> {
    executable
        .functions
        .get(usize::from(id.get()))
        .ok_or_else(|| {
            ValidationError::executable(ValidationErrorKind::TableReference {
                table: "functions",
                operand,
                actual: u64::from(id.get()),
                length: executable.functions.len(),
            })
        })
}

#[expect(
    clippy::too_many_arguments,
    reason = "verifier context keeps diagnostics actionable"
)]
pub(super) fn validate_layout_operand(
    executable: &crate::Executable,
    function_id: FunctionId,
    function: &FunctionInfo,
    address: InstructionAddress,
    opcode: Opcode,
    a: u16,
    b: u16,
    c: u16,
    auxiliary: u8,
) -> Result<bool, ValidationError> {
    match opcode {
        Opcode::MakeRecord => {
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            let Some(layout) = executable
                .records
                .get(usize::from(RecordTypeId::new(b).get()))
            else {
                return Err(table_error(
                    executable,
                    function_id,
                    address,
                    opcode,
                    "record layouts",
                    "record type",
                    b,
                    executable.records.len(),
                ));
            };
            canonical(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "record value window",
                c,
                layout.fields.len(),
            )?;
            Ok(true)
        }
        Opcode::LoadField | Opcode::StoreField => {
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                if opcode == Opcode::LoadField {
                    "destination"
                } else {
                    "record"
                },
                a,
            )?;
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                if opcode == Opcode::LoadField {
                    "record"
                } else {
                    "value"
                },
                if opcode == Opcode::LoadField { b } else { c },
            )?;
            let field = if opcode == Opcode::LoadField { c } else { b };
            let available = executable
                .records
                .iter()
                .map(|layout| layout.fields.len())
                .max()
                .unwrap_or(0);
            if usize::from(field) >= available {
                return Err(ValidationError::instruction(
                    executable,
                    function_id,
                    address,
                    Some(opcode),
                    ValidationErrorKind::LayoutReference {
                        operand: "record field",
                        actual: field,
                        available,
                    },
                ));
            }
            canonical(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )?;
            Ok(true)
        }
        Opcode::UpdateRecord => {
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "record",
                a,
            )?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "record override window",
                b,
                usize::from(c) * 2,
            )?;
            canonical(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )?;
            Ok(true)
        }
        Opcode::MakeEnum => {
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            let Some(variant) = executable
                .enum_variants
                .get(usize::from(EnumVariantId::new(b).get()))
            else {
                return Err(table_error(
                    executable,
                    function_id,
                    address,
                    opcode,
                    "enum variants",
                    "enum variant",
                    b,
                    executable.enum_variants.len(),
                ));
            };
            canonical(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )?;
            validate_window(
                executable,
                function_id,
                function,
                address,
                opcode,
                "enum value window",
                c,
                variant.fields.len(),
            )?;
            Ok(true)
        }
        Opcode::TestVariant => {
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "enum value",
                b,
            )?;
            if usize::from(c) >= executable.enum_variants.len() {
                return Err(table_error(
                    executable,
                    function_id,
                    address,
                    opcode,
                    "enum variants",
                    "enum variant",
                    c,
                    executable.enum_variants.len(),
                ));
            }
            canonical(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )?;
            Ok(true)
        }
        Opcode::LoadEnumField => {
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "destination",
                a,
            )?;
            super::instruction::validate_register(
                executable,
                function_id,
                function,
                address,
                opcode,
                "enum value",
                b,
            )?;
            let available = executable
                .enum_variants
                .iter()
                .map(|variant| variant.fields.len())
                .max()
                .unwrap_or(0);
            if usize::from(c) >= available {
                return Err(ValidationError::instruction(
                    executable,
                    function_id,
                    address,
                    Some(opcode),
                    ValidationErrorKind::LayoutReference {
                        operand: "enum field",
                        actual: c,
                        available,
                    },
                ));
            }
            canonical(
                executable,
                function_id,
                address,
                opcode,
                "auxiliary",
                auxiliary,
                0,
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn canonical(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    operand: &'static str,
    actual: u8,
    expected: u8,
) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::instruction(
            executable,
            function_id,
            address,
            Some(opcode),
            ValidationErrorKind::NonCanonicalOperand {
                operand,
                actual: u64::from(actual),
                expected: u64::from(expected),
            },
        ))
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "verifier context keeps diagnostics actionable"
)]
fn table_error(
    executable: &crate::Executable,
    function_id: FunctionId,
    address: InstructionAddress,
    opcode: Opcode,
    table: &'static str,
    operand: &'static str,
    actual: u16,
    length: usize,
) -> ValidationError {
    ValidationError::instruction(
        executable,
        function_id,
        address,
        Some(opcode),
        ValidationErrorKind::TableReference {
            table,
            operand,
            actual: u64::from(actual),
            length,
        },
    )
}
