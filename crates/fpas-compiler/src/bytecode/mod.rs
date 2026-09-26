//! Typed IR to verified bytecode construction.

mod allocation;
mod blocks;
mod debug;
mod function;
mod metadata;
mod selection;
mod superinstructions;

use fpas_bytecode::{Executable, FunctionId};
use fpas_ir::Program;

use crate::CompileError;
use crate::error::internal_compiler_error;

use self::debug::compile_debug_types;
use self::function::compile_function;
use self::metadata::MetadataBuilder;

pub(crate) use self::superinstructions::integer_immediate;

pub(super) fn compile_program(
    mut program: Program,
) -> Result<fpas_bytecode::VerifiedExecutable, CompileError> {
    program
        .validate()
        .map_err(|error| compile_error(&error.to_string()))?;
    crate::optimize::optimize(&mut program);
    let program = &program;
    if program.entry != fpas_ir::FunctionId::new(0) || program.functions.is_empty() {
        return Err(compile_error(
            "register root function must use dense function identifier zero",
        ));
    }
    let (mut metadata, _) = MetadataBuilder::new(&program.functions[0].name)?;
    let mut code = Vec::new();
    let mut functions = Vec::with_capacity(program.functions.len());
    let mut instruction_addresses = Vec::with_capacity(program.functions.len());
    for (index, function) in program.functions.iter().enumerate() {
        if usize::try_from(function.id.get()).ok() != Some(index) {
            return Err(compile_error(
                "register function identifiers must be dense and ordered",
            ));
        }
        let (compiled, addresses) = compile_function(program, function, &mut code, &mut metadata)?;
        functions.push(compiled);
        instruction_addresses.push(addresses);
    }
    let globals = program
        .globals
        .iter()
        .map(|global| {
            let name = metadata.intern_string(&global.name)?;
            let initializer = global
                .initializer
                .map(|initializer| {
                    let function_index = usize::try_from(initializer.function.get())
                        .map_err(|_| compile_error("global initializer function overflow"))?;
                    let instruction = instruction_addresses
                        .get(function_index)
                        .and_then(|addresses| {
                            addresses.iter().find(|point| {
                                point.emitted
                                    && point.block == initializer.location.block
                                    && point.instruction == initializer.location.instruction
                            })
                        })
                        .map(|point| point.address)
                        .ok_or_else(|| {
                            compile_error("global initializer has no emitted store instruction")
                        })?;
                    let function = u16::try_from(initializer.function.get())
                        .map(fpas_bytecode::FunctionId::new)
                        .map_err(|_| compile_error("global initializer function overflow"))?;
                    Ok(fpas_bytecode::GlobalInitializer {
                        function,
                        instruction,
                    })
                })
                .transpose()?;
            Ok(fpas_bytecode::GlobalInfo {
                name,
                ty: fpas_bytecode::DebugTypeId::new(global.ty.get()),
                mutable: global.mutable,
                initializer,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let records = program
        .record_layouts
        .iter()
        .map(|layout| {
            let name = metadata.intern_string(&layout.name)?;
            let fields = layout
                .fields
                .iter()
                .map(|field| {
                    metadata
                        .intern_string(&field.name)
                        .map(|name| fpas_bytecode::RecordField {
                            name,
                            ty: fpas_bytecode::DebugTypeId::new(field.ty.get()),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let properties = layout
                .properties
                .iter()
                .map(|property| {
                    Ok(fpas_bytecode::RecordProperty {
                        name: metadata.intern_string(&property.name)?,
                        getter: metadata.intern_string(&property.getter)?,
                    })
                })
                .collect::<Result<Vec<_>, CompileError>>()?;
            let methods = layout
                .methods
                .iter()
                .map(|method| {
                    Ok(fpas_bytecode::RecordMethod {
                        name: metadata.intern_string(&method.name)?,
                        routine: metadata.intern_string(&method.routine)?,
                    })
                })
                .collect::<Result<Vec<_>, CompileError>>()?;
            Ok(fpas_bytecode::RecordLayout {
                name,
                fields,
                properties,
                methods,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let mut enums = Vec::new();
    let mut enum_variants = Vec::new();
    for layout in &program.enum_layouts {
        enums.push(fpas_bytecode::EnumLayout {
            name: metadata.intern_string(&layout.name)?,
        });
        for variant in &layout.variants {
            let fields = variant
                .field_names
                .iter()
                .map(|name| metadata.intern_string(name))
                .collect::<Result<Vec<_>, _>>()?;
            enum_variants.push(fpas_bytecode::EnumVariant {
                owner: fpas_bytecode::EnumTypeId::new(
                    u16::try_from(layout.id.get())
                        .map_err(|_| compile_error("enum layout exceeds u16"))?,
                ),
                name: metadata.intern_string(&variant.name)?,
                fields,
                field_types: variant
                    .fields
                    .iter()
                    .map(|ty| fpas_bytecode::DebugTypeId::new(ty.get()))
                    .collect(),
            });
        }
    }
    let (constants, strings, source_map) = metadata.finish();
    let debug_types = compile_debug_types(program)?;
    let executable = Executable {
        code,
        functions,
        constants,
        strings,
        globals,
        records,
        enums,
        enum_variants,
        debug_types,
        source_map,
        entry: FunctionId::new(0),
    };
    executable.verify().map_err(|error| {
        compile_error(&format!(
            "generated executable failed verification: {error}"
        ))
    })
}

pub(super) fn compile_error(message: &str) -> CompileError {
    internal_compiler_error(
        format!("Register bytecode construction failed: {message}."),
        "This is an internal compiler error. Re-run compilation and report the source program.",
        1,
        1,
    )
}
