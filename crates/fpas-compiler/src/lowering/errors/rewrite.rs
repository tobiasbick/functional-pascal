//! Materialize the hidden constructor ABI after the transitive requirement analysis.
//! Documentation: `docs/pascal/language/types/errors.md`.

mod bindings;

use super::*;
use fpas_ir::{IntrinsicId, IrType, ValueDefinition};

pub(super) fn rewrite(
    functions: &mut Vec<Function>,
    types: &mut TypeTable,
    parameters: &Parameters,
    requirements: &Requirements,
    metadata: &fpas_sema::AnalysisMetadata,
    span: fpas_lexer::Span,
    globals: &mut [fpas_ir::Global],
) -> Result<(), CompileError> {
    let original = functions.len();
    for index in 0..original {
        let mut function = functions[index].clone();
        let mut next = next_value(&function)?;
        let mut hidden = BTreeMap::new();
        let mut locations = BTreeMap::new();
        for request in requirements.get(&function.id).into_iter().flatten() {
            let ty = instances::factory_type(types, request, span)?;
            let value = value(&mut next, ty)?;
            hidden.insert(request.clone(), value.id);
            function.signature.parameters.push(ty);
            function.parameters.push(value);
        }
        for block_index in 0..function.blocks.len() {
            let old = std::mem::take(&mut function.blocks[block_index].instructions);
            let mut rewritten = Vec::new();
            for (old_index, mut instruction) in old.into_iter().enumerate() {
                let source = instruction.source;
                match instruction.operation.clone() {
                    Operation::MakeEnum {
                        layout,
                        variant,
                        fields,
                    } if types.errors.layouts.contains_key(&layout)
                        && types.errors.concrete(layout).is_none() =>
                    {
                        let request = Request {
                            instance: construction_instance(
                                &instruction,
                                layout,
                                function.id,
                                parameters,
                                metadata,
                                types,
                            )?,
                            variant: u16::try_from(variant.get())
                                .map_err(|_| error("error variant overflow"))?,
                        };
                        let callee = hidden
                            .get(&request)
                            .copied()
                            .ok_or_else(|| error("missing hidden error constructor"))?;
                        function.max_call_arguments =
                            function.max_call_arguments.max(fields.len() as u32);
                        instruction.operation = Operation::CallValue {
                            callee,
                            arguments: fields,
                        };
                    }
                    Operation::CallDirect {
                        function: target,
                        mut arguments,
                    } => {
                        for request in requirements.get(&target).into_iter().flatten() {
                            let request = apply(
                                request,
                                &instruction,
                                function.id,
                                parameters,
                                metadata,
                                types,
                            )?;
                            arguments.push(factory_value(
                                functions,
                                types,
                                &request,
                                &hidden,
                                &mut rewritten,
                                &mut next,
                                source,
                                span,
                            )?);
                        }
                        function.max_call_arguments =
                            function.max_call_arguments.max(arguments.len() as u32);
                        instruction.operation = Operation::CallDirect {
                            function: target,
                            arguments,
                        };
                    }
                    Operation::MakeClosure {
                        function: target,
                        captures,
                    } if requirements
                        .get(&target)
                        .is_some_and(|requests| !requests.is_empty()) =>
                    {
                        let mut supplied = Vec::new();
                        for request in &requirements[&target] {
                            let request = apply(
                                request,
                                &instruction,
                                function.id,
                                parameters,
                                metadata,
                                types,
                            )?;
                            supplied.push(factory_value(
                                functions,
                                types,
                                &request,
                                &hidden,
                                &mut rewritten,
                                &mut next,
                                source,
                                span,
                            )?);
                        }
                        instruction.operation = bindings::bind(
                            functions,
                            types,
                            &mut function,
                            &instruction,
                            target,
                            captures,
                            supplied,
                            &mut rewritten,
                            &mut next,
                            span,
                        )?;
                    }
                    Operation::TestVariant {
                        value: tested,
                        layout,
                        variant,
                    } if types.errors.layouts.contains_key(&layout)
                        && types.errors.concrete(layout).is_none() =>
                    {
                        let enumeration = &types.errors.layouts[&layout];
                        let name = format!(
                            "{}.{}",
                            types.errors.display_name(enumeration),
                            enumeration.variants[variant.get() as usize].name
                        );
                        let expected = value(&mut next, types::STRING)?;
                        rewritten.push(Instruction {
                            source,
                            result: Some(expected),
                            operation: Operation::Const(fpas_ir::Constant::String(name)),
                        });
                        instruction.operation = Operation::Intrinsic {
                            intrinsic: IntrinsicId::new(u32::from(u16::from(
                                fpas_bytecode::Intrinsic::Conv(
                                    fpas_bytecode::ConvIntrinsic::ErrorMatchesVariant,
                                ),
                            ))),
                            arguments: vec![tested, expected.id],
                        };
                        function.max_call_arguments = function.max_call_arguments.max(2);
                    }
                    _ => {}
                }
                locations.insert(
                    (function.blocks[block_index].id, old_index),
                    rewritten.len(),
                );
                rewritten.push(instruction);
            }
            function.blocks[block_index].instructions = rewritten;
        }
        for point in &mut function.debug.sequence_points {
            if let Some(&updated) = locations.get(&(point.block, point.instruction)) {
                point.instruction = updated;
            }
        }
        for initializer in function
            .debug
            .bindings
            .iter_mut()
            .filter_map(|binding| binding.initializer.as_mut())
        {
            if let Some(&updated) = locations.get(&(initializer.block, initializer.instruction)) {
                initializer.instruction = updated;
            }
        }
        for initializer in globals
            .iter_mut()
            .filter_map(|global| global.initializer.as_mut())
            .filter(|initializer| initializer.function == function.id)
        {
            if let Some(&updated) =
                locations.get(&(initializer.location.block, initializer.location.instruction))
            {
                initializer.location.instruction = updated;
            }
        }
        functions[index] = function;
    }
    let templates = types
        .enum_layouts
        .iter()
        .filter(|layout| layout.error_template)
        .map(|layout| layout.id)
        .collect::<BTreeSet<_>>();
    for definition in &mut types.definitions {
        if matches!(definition.kind, IrType::Enum(id) if templates.contains(&id)) {
            definition.kind = IrType::Dynamic;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn factory_value(
    functions: &mut Vec<Function>,
    types: &mut TypeTable,
    request: &Request,
    hidden: &BTreeMap<Request, ValueId>,
    instructions: &mut Vec<Instruction>,
    next: &mut u32,
    source: Option<fpas_ir::SourceSpan>,
    span: fpas_lexer::Span,
) -> Result<ValueId, CompileError> {
    if let Some(value) = hidden.get(request) {
        return Ok(*value);
    }
    request.instance.validate(0).map_err(error)?;
    let (function, ty) = instances::factory(functions, types, request, span)?;
    let result = value(next, ty)?;
    instructions.push(Instruction {
        source,
        result: Some(result),
        operation: Operation::MakeClosure {
            function,
            captures: vec![],
        },
    });
    Ok(result.id)
}

fn next_value(function: &Function) -> Result<u32, CompileError> {
    function
        .parameters
        .iter()
        .map(|value| value.id.get())
        .chain(
            function
                .blocks
                .iter()
                .flat_map(|block| block.parameters.iter().map(|parameter| parameter.id.get())),
        )
        .chain(function.blocks.iter().flat_map(|block| {
            block
                .instructions
                .iter()
                .filter_map(|instruction| instruction.result.map(|value| value.id.get()))
        }))
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| error("error ABI value identifier overflow"))
}

fn value(next: &mut u32, ty: TypeId) -> Result<ValueDefinition, CompileError> {
    let id = ValueId::new(*next);
    *next = next
        .checked_add(1)
        .ok_or_else(|| error("error ABI value identifier overflow"))?;
    Ok(ValueDefinition { id, ty })
}
