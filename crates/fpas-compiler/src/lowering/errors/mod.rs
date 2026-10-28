//! Constructor requirements through erased generic bodies and ordinary callables.
//! Documentation: `docs/pascal/language/types/errors.md`.

mod instances;
mod rewrite;

use super::{
    routines::Routine,
    types::{self, TypeTable},
};
use crate::{CompileError, errors::error};
use fpas_ir::{BlockId, Function, FunctionId, Instruction, Operation, TypeId, ValueId};
use fpas_sema::{GenericParamDef, Ty, TypeConstraint};
use fpas_unit::errors::{ErrorConstructorRequest as Request, ErrorConstructorSignature};
use std::collections::{BTreeMap, BTreeSet};

type Parameters = BTreeMap<FunctionId, Vec<GenericParamDef>>;
type Requirements = BTreeMap<FunctionId, BTreeSet<Request>>;

pub(super) fn prepare(
    types: &mut TypeTable,
    declarations: &[fpas_parser::Decl],
    interfaces: &[fpas_unit::interface::UnitInterface],
) -> Result<(), CompileError> {
    types.errors.imported_equality = fpas_unit::equality::EqualityDeclarations::new(
        interfaces
            .iter()
            .flat_map(|interface| &interface.errors.equality_declarations)
            .cloned(),
    )
    .map_err(error)?
    .into_declarations();
    for schema in interfaces
        .iter()
        .flat_map(|interface| &interface.errors.schemas)
    {
        types
            .errors
            .schemas
            .insert(schema.declaration.clone(), schema.clone());
    }
    for ty in types.errors.named.values() {
        if let Ty::Enum(enumeration) = ty
            && enumeration.is_error
            && enumeration.type_args.is_empty()
        {
            let schema = fpas_sema::error_schema(enumeration, &types.errors.owner, &|name| {
                types.errors.named.get(&name.to_ascii_lowercase()).cloned()
            })
            .map_err(error)?;
            types
                .errors
                .schemas
                .entry(schema.declaration.clone())
                .or_insert(schema);
        }
    }
    for declaration in declarations {
        if let fpas_parser::Decl::TypeDef(definition) = declaration
            && let fpas_parser::TypeBody::Enum(body) = &definition.body
            && body.is_error
        {
            types.errors.locations.insert(
                format!("{}.{}", types.errors.owner, definition.name).to_ascii_lowercase(),
                definition.span,
            );
        }
    }
    Ok(())
}

pub(super) struct LoweringInput<'a, 'r> {
    pub routines: &'a [Routine<'r>],
    pub metadata: &'a fpas_sema::AnalysisMetadata,
    pub interfaces: &'a [fpas_unit::interface::UnitInterface],
    pub span: fpas_lexer::Span,
    pub owners: &'a std::collections::HashMap<FunctionId, FunctionId>,
}

pub(super) fn lower(
    functions: &mut Vec<Function>,
    types: &mut TypeTable,
    globals: &mut [fpas_ir::Global],
    input: LoweringInput<'_, '_>,
) -> Result<(), CompileError> {
    let LoweringInput {
        routines,
        metadata,
        interfaces,
        span,
        owners,
    } = input;
    if types.errors.schemas.is_empty() {
        return Ok(());
    }
    let mut parameters = Parameters::new();
    for (index, routine) in routines.iter().enumerate() {
        let declared = metadata
            .declaration_types
            .get(&(routine.span().source_id, routine.span().offset));
        let mut params = match declared {
            Some(Ty::Function(function)) => function.type_params.clone(),
            Some(Ty::Procedure(procedure)) => procedure.type_params.clone(),
            _ => routine
                .type_params()
                .iter()
                .map(|parameter| GenericParamDef {
                    name: parameter.name.clone(),
                    constraint: parameter
                        .constraint
                        .as_deref()
                        .and_then(TypeConstraint::from_name),
                })
                .collect(),
        };
        if let Routine::RecordFunction(owner, _) | Routine::RecordProcedure(owner, _) = routine {
            let mut inherited = owner
                .type_params
                .iter()
                .map(|parameter| GenericParamDef {
                    name: parameter.name.clone(),
                    constraint: parameter
                        .constraint
                        .as_deref()
                        .and_then(TypeConstraint::from_name),
                })
                .collect::<Vec<_>>();
            params.retain(|parameter| {
                !inherited
                    .iter()
                    .any(|owner| owner.name.eq_ignore_ascii_case(&parameter.name))
            });
            inherited.extend(params);
            params = inherited;
        }
        parameters.insert(FunctionId::new(index as u32 + 1), params);
    }
    let own_parameters = parameters.clone();
    for function in functions.iter() {
        let mut inherited = owners
            .get(&function.id)
            .and_then(|owner| parameters.get(owner))
            .cloned()
            .unwrap_or_default();
        for parameter in own_parameters.get(&function.id).into_iter().flatten() {
            if !inherited
                .iter()
                .any(|existing| existing.name.eq_ignore_ascii_case(&parameter.name))
            {
                inherited.push(parameter.clone());
            }
        }
        parameters.insert(function.id, inherited);
    }
    let mut requirements = Requirements::new();
    for function in functions.iter() {
        if let Some(routine) = interfaces
            .iter()
            .flat_map(|interface| &interface.errors.routines)
            .find(|routine| routine.routine.eq_ignore_ascii_case(&function.name))
        {
            requirements.insert(
                function.id,
                routine.signature.requirements.iter().cloned().collect(),
            );
            parameters.insert(
                function.id,
                (0..routine.signature.type_parameters)
                    .map(|index| GenericParamDef {
                        name: format!("$import{index}"),
                        constraint: None,
                    })
                    .collect(),
            );
        }
        for instruction in function.blocks.iter().flat_map(|block| &block.instructions) {
            if let Operation::MakeEnum {
                layout, variant, ..
            } = instruction.operation
                && let Some(enumeration) = types.errors.layouts.get(&layout)
                && types.errors.concrete(layout).is_none()
            {
                let _ = enumeration;
                let request = Request {
                    instance: construction_instance(
                        instruction,
                        layout,
                        function.id,
                        &parameters,
                        metadata,
                        types,
                    )?,
                    variant: u16::try_from(variant.get())
                        .map_err(|_| error("error variant overflow"))?,
                };
                requirements.entry(function.id).or_default().insert(request);
            }
        }
    }
    let mut converged = false;
    for _ in 0..64 {
        let previous = requirements.clone();
        for function in functions.iter() {
            for instruction in function.blocks.iter().flat_map(|block| &block.instructions) {
                let target = match instruction.operation {
                    Operation::CallDirect { function, .. }
                    | Operation::MakeClosure { function, .. } => function,
                    _ => continue,
                };
                for request in previous.get(&target).into_iter().flatten() {
                    // Generated constructor bodies already use their lexical owner's
                    // parameter indices. Source constructor arguments are not routine
                    // arguments of the generated body and must not substitute them again.
                    let constructor = functions.iter().any(|candidate| {
                        candidate.id == target && candidate.name.starts_with("$constructor_")
                    });
                    let applied = if constructor {
                        request.clone()
                    } else {
                        apply(
                            request,
                            instruction,
                            function.id,
                            &parameters,
                            metadata,
                            types,
                        )?
                    };
                    if applied.instance.validate(0).is_err() {
                        requirements.entry(function.id).or_default().insert(applied);
                    }
                }
            }
        }
        if previous == requirements {
            converged = true;
            break;
        }
        if requirements.values().map(BTreeSet::len).sum::<usize>() > 4096 {
            return Err(error(
                "error constructor requirements exceed the finite call-graph limit",
            ));
        }
    }
    if !converged {
        return Err(error(
            "error constructor requirements do not have a finite stable instance set",
        ));
    }
    for (&id, requests) in &requirements {
        let signature = ErrorConstructorSignature {
            type_parameters: u16::try_from(parameters.get(&id).map(Vec::len).unwrap_or(0))
                .map_err(|_| error("too many error type parameters"))?,
            requirements: requests.iter().cloned().collect(),
        };
        signature.validate().map_err(error)?;
        types.errors.requirements.insert(id, signature);
    }
    rewrite::rewrite(
        functions,
        types,
        &parameters,
        &requirements,
        metadata,
        span,
        globals,
    )?;
    Ok(())
}

fn apply(
    request: &Request,
    instruction: &Instruction,
    caller: FunctionId,
    parameters: &Parameters,
    metadata: &fpas_sema::AnalysisMetadata,
    types: &TypeTable,
) -> Result<Request, CompileError> {
    let key = instruction
        .source
        .map(|span| (span.source_id(), span.offset()));
    let caller_parameters = parameters
        .get(&caller)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let routine_arguments = key
        .and_then(|key| metadata.routine_type_arguments.get(&key))
        .map(Vec::as_slice)
        .unwrap_or_default();
    let owner_arguments = key
        .and_then(|key| metadata.method_owner_type_arguments.get(&key))
        .map(Vec::as_slice)
        .unwrap_or_default();
    let arguments = owner_arguments
        .iter()
        .chain(routine_arguments)
        .cloned()
        .collect::<Vec<_>>();
    let target = match instruction.operation {
        Operation::CallDirect { function, .. } | Operation::MakeClosure { function, .. } => {
            Some(function)
        }
        _ => None,
    };
    let target_parameters = target
        .and_then(|target| parameters.get(&target))
        .map(Vec::as_slice)
        .unwrap_or_default();
    let inherited = target_parameters.len().saturating_sub(arguments.len());
    let arguments = target_parameters[..inherited]
        .iter()
        .map(|parameter| Ty::GenericParam(parameter.name.clone(), parameter.constraint))
        .chain(arguments.iter().cloned())
        .collect::<Vec<_>>();
    let arguments = arguments
        .iter()
        .map(|argument| {
            types
                .errors
                .identity(argument, caller_parameters)
                .map_err(error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    request
        .substitute(
            &arguments,
            u16::try_from(caller_parameters.len())
                .map_err(|_| error("too many error type parameters"))?,
        )
        .map_err(error)
}

fn construction_instance(
    instruction: &Instruction,
    layout: fpas_ir::EnumLayoutId,
    caller: FunctionId,
    parameters: &Parameters,
    metadata: &fpas_sema::AnalysisMetadata,
    types: &TypeTable,
) -> Result<fpas_code::ErrorInstanceKey, CompileError> {
    let semantic = instruction.source.and_then(|span| {
        metadata
            .source_expr_types
            .get(&(span.source_id(), span.offset(), span.length()))
    });
    let enumeration = match semantic {
        Some(Ty::Enum(enumeration)) if enumeration.is_error => enumeration.as_ref(),
        Some(Ty::Function(function)) => match function.return_type.as_ref() {
            Ty::Enum(enumeration) if enumeration.is_error => enumeration.as_ref(),
            _ => &types.errors.layouts[&layout],
        },
        _ => &types.errors.layouts[&layout],
    };
    types
        .errors
        .instance(
            enumeration,
            parameters
                .get(&caller)
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )
        .map_err(error)
}
