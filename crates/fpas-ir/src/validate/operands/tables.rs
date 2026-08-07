use std::collections::{BTreeMap, BTreeSet};

use crate::instruction::{TypeCategory, binary_categories};
use crate::{
    BasicBlock, BinaryOperation, BlockId, Constant, EnumLayoutId, Function, FunctionId, IrType,
    Operation, Program, RecordLayoutId, Terminator, TypeId, ValueDefinition, ValueId,
};

use super::{EntityKind, ValidationError, ValidationErrorKind, function_error, program_error};

/// Validates every program-wide deterministic table before function operands are inspected.
pub(crate) fn validate_program_tables(program: &Program) -> Result<(), ValidationError> {
    validate_unique(
        program.types.iter().map(|item| item.id.get()),
        EntityKind::Type,
    )?;
    validate_unique(
        program.globals.iter().map(|item| item.id.get()),
        EntityKind::Global,
    )?;
    validate_unique(
        program.record_layouts.iter().map(|item| item.id.get()),
        EntityKind::RecordLayout,
    )?;
    validate_unique(
        program.enum_layouts.iter().map(|item| item.id.get()),
        EntityKind::EnumLayout,
    )?;
    validate_unique(
        program.intrinsics.iter().map(|item| item.id.get()),
        EntityKind::Intrinsic,
    )?;
    validate_unique(
        program.functions.iter().map(|item| item.id.get()),
        EntityKind::Function,
    )?;

    for definition in &program.types {
        validate_ir_type(program, &definition.kind)?;
    }
    for global in &program.globals {
        require_type(program, global.ty)?;
    }
    for layout in &program.record_layouts {
        validate_unique(
            layout.fields.iter().map(|field| field.id.get()),
            EntityKind::Field,
        )?;
        for field in &layout.fields {
            require_type(program, field.ty)?;
        }
    }
    for layout in &program.enum_layouts {
        validate_unique(
            layout.variants.iter().map(|variant| variant.id.get()),
            EntityKind::Variant,
        )?;
        for variant in &layout.variants {
            for ty in &variant.fields {
                require_type(program, *ty)?;
            }
        }
    }
    for intrinsic in &program.intrinsics {
        validate_signature_types(program, &intrinsic.parameters, intrinsic.result)?;
    }
    Ok(())
}

/// Validates all operand and result invariants for one function.
pub(crate) fn validate_function(
    program: &Program,
    function: &Function,
) -> Result<(), ValidationError> {
    validate_signature_types(
        program,
        &function.signature.parameters,
        function.signature.result,
    )?;
    if function.parameters.len() != function.signature.parameters.len() {
        return Err(function_error(
            function.id,
            None,
            None,
            ValidationErrorKind::DirectCallSignature {
                expected: function.signature.parameters.len(),
                actual: function.parameters.len(),
            },
        ));
    }
    validate_unique_locals(function)?;
    validate_function_declarations(program, function)?;
    let all_values = collect_value_definitions(program, function)?;
    let parameter_values = function
        .parameters
        .iter()
        .map(|value| value.id)
        .collect::<BTreeSet<_>>();

    for block in &function.blocks {
        validate_block(program, function, block, &all_values, &parameter_values)?;
    }
    Ok(())
}

fn validate_function_declarations(
    program: &Program,
    function: &Function,
) -> Result<(), ValidationError> {
    for (parameter, expected) in function
        .parameters
        .iter()
        .zip(function.signature.parameters.iter())
    {
        require_type_at(program, function.id, None, None, parameter.ty)?;
        if parameter.ty != *expected {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::OperandType {
                    operand: "function parameter",
                    expected: expected.get(),
                    actual: parameter.ty.get(),
                },
            ));
        }
    }
    for local in &function.locals {
        require_type_at(program, function.id, None, None, local.ty)?;
    }
    for capture in &function.captures {
        require_type_at(program, function.id, None, None, capture.ty)?;
    }
    for block in &function.blocks {
        for parameter in &block.parameters {
            require_type_at(program, function.id, Some(block.id), None, parameter.ty)?;
        }
    }
    Ok(())
}

fn validate_unique_locals(function: &Function) -> Result<(), ValidationError> {
    let mut seen = BTreeSet::new();
    for local in &function.locals {
        if !seen.insert(local.id) {
            return Err(function_error(
                function.id,
                None,
                None,
                ValidationErrorKind::DuplicateId {
                    entity: EntityKind::Local,
                    id: local.id.get(),
                },
            ));
        }
    }
    Ok(())
}

fn collect_value_definitions(
    program: &Program,
    function: &Function,
) -> Result<BTreeMap<ValueId, TypeId>, ValidationError> {
    let mut definitions = BTreeMap::new();
    for definition in &function.parameters {
        insert_value(program, function, None, None, definition, &mut definitions)?;
    }
    for block in &function.blocks {
        for parameter in &block.parameters {
            insert_value(
                program,
                function,
                Some(block.id),
                None,
                &ValueDefinition {
                    id: parameter.id,
                    ty: parameter.ty,
                },
                &mut definitions,
            )?;
        }
        for (instruction_index, instruction) in block.instructions.iter().enumerate() {
            if let Some(result) = instruction.result {
                insert_value(
                    program,
                    function,
                    Some(block.id),
                    Some(instruction_index),
                    &result,
                    &mut definitions,
                )?;
            }
        }
    }
    Ok(definitions)
}

fn insert_value(
    program: &Program,
    function: &Function,
    block: Option<BlockId>,
    instruction: Option<usize>,
    definition: &ValueDefinition,
    definitions: &mut BTreeMap<ValueId, TypeId>,
) -> Result<(), ValidationError> {
    require_type_at(program, function.id, block, instruction, definition.ty)?;
    if definitions.insert(definition.id, definition.ty).is_some() {
        return Err(function_error(
            function.id,
            block,
            instruction,
            ValidationErrorKind::DuplicateId {
                entity: EntityKind::Value,
                id: definition.id.get(),
            },
        ));
    }
    Ok(())
}
