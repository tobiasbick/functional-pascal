//! Bind hidden constructors while preserving the source callable's arity and captures.
//! Documentation: `docs/pascal/language/types/errors.md`.

use super::*;
use fpas_ir::{
    BasicBlock, CaptureDeclaration, CaptureKind, DebugBinding, DebugBindingId, DebugBindingKind,
    DebugCaptureSource, DebugScope, FunctionDebugInfo, FunctionSignature, Local, LocalId,
    Terminator,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn bind(
    functions: &mut Vec<Function>,
    max_function_id: &mut u32,
    types: &mut TypeTable,
    owner: &mut Function,
    instruction: &Instruction,
    target: FunctionId,
    target_function: &TargetFunction,
    captures: Vec<ValueId>,
    factories: Vec<ValueId>,
    prefix: &mut Vec<Instruction>,
    next: &mut u32,
    span: fpas_lexer::Span,
) -> Result<Operation, CompileError> {
    let visible = instruction
        .result
        .ok_or_else(|| error("bound error routine has no result"))?
        .ty;
    let Some(IrType::Function { parameters, result }) = types.kind(visible).cloned() else {
        return Err(error("bound error routine has no callable signature"));
    };
    let mut capture_types = Vec::new();
    for factory in &factories {
        let ty = owner
            .parameters
            .iter()
            .find(|parameter| parameter.id == *factory)
            .map(|parameter| parameter.ty)
            .or_else(|| {
                prefix.iter().find_map(|instruction| {
                    instruction
                        .result
                        .filter(|value| value.id == *factory)
                        .map(|value| value.ty)
                })
            })
            .ok_or_else(|| error("bound hidden constructor type is unavailable"))?;
        capture_types.push(ty);
    }
    let mut all_captures = Vec::new();
    let original_count = usize::from(!captures.is_empty());
    if !captures.is_empty() {
        let mut callable_parameters = parameters.clone();
        callable_parameters.extend_from_slice(&capture_types);
        let ty = types.function_type(callable_parameters, target_function.result, span)?;
        let callee = value(next, ty)?;
        prefix.push(Instruction {
            source: instruction.source,
            result: Some(callee),
            operation: Operation::MakeClosure {
                function: target,
                captures,
            },
        });
        all_captures.push(callee.id);
        capture_types.insert(0, ty);
    }
    all_captures.extend(factories);
    let mut sources = Vec::new();
    if owner.debug.scopes.is_empty() {
        owner.debug.scopes.push(DebugScope {
            id: 0,
            parent: None,
        });
    }
    for (&captured, &ty) in all_captures.iter().zip(&capture_types) {
        let local =
            LocalId::try_from_index(owner.locals.len()).map_err(|e| error(e.to_string()))?;
        owner.locals.push(Local {
            id: local,
            ty,
            mutable: false,
            capture: None,
        });
        let binding = DebugBindingId::try_from_index(owner.debug.bindings.len())
            .map_err(|e| error(e.to_string()))?;
        owner.debug.bindings.push(DebugBinding {
            local,
            name: "$error.bound".into(),
            type_name: None,
            projection: None,
            kind: DebugBindingKind::Local,
            ty,
            mutable: false,
            scope: 0,
            declaration: None,
            hidden: true,
            cell_backed: false,
            initializer: None,
        });
        prefix.push(Instruction {
            source: instruction.source,
            result: None,
            operation: Operation::WriteLocal {
                value: captured,
                local,
            },
        });
        sources.push(DebugCaptureSource {
            binding,
            ty,
            kind: CaptureKind::Value,
        });
    }
    *max_function_id = max_function_id
        .checked_add(1)
        .ok_or_else(|| error("bound error function identifier overflow"))?;
    let id = FunctionId::new(*max_function_id);
    let params = parameters
        .iter()
        .enumerate()
        .map(|(index, &ty)| {
            Ok(ValueDefinition {
                id: ValueId::try_from_index(index).map_err(|e| error(e.to_string()))?,
                ty,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let mut locals = parameters
        .iter()
        .enumerate()
        .map(|(index, &ty)| Local {
            id: LocalId::new(index as u32),
            ty,
            mutable: false,
            capture: None,
        })
        .collect::<Vec<_>>();
    let mut body = Vec::new();
    let mut next_bound =
        u32::try_from(parameters.len()).map_err(|_| error("bound error arity overflow"))?;
    let mut retained = Vec::new();
    let mut debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        lexical_owner: Some(owner.id),
        capture_sources: sources,
        ..Default::default()
    };
    for (index, &ty) in capture_types.iter().enumerate() {
        let local = LocalId::try_from_index(locals.len()).map_err(|e| error(e.to_string()))?;
        locals.push(Local {
            id: local,
            ty,
            mutable: false,
            capture: Some(CaptureKind::Value),
        });
        debug.bindings.push(DebugBinding {
            local,
            name: format!("$error.bound{index}"),
            type_name: None,
            projection: None,
            kind: DebugBindingKind::Capture,
            ty,
            mutable: false,
            scope: 0,
            declaration: None,
            hidden: true,
            cell_backed: false,
            initializer: None,
        });
        let read = value(&mut next_bound, ty)?;
        body.push(Instruction {
            source: instruction.source,
            result: Some(read),
            operation: Operation::ReadLocal(local),
        });
        retained.push(read.id);
    }
    let mut arguments = params
        .iter()
        .map(|parameter| parameter.id)
        .collect::<Vec<_>>();
    arguments.extend_from_slice(&retained[original_count..]);
    let operation = if original_count == 0 {
        Operation::CallDirect {
            function: target,
            arguments: arguments.clone(),
        }
    } else {
        Operation::CallValue {
            callee: retained[0],
            arguments: arguments.clone(),
        }
    };
    let returned = value(&mut next_bound, result)?;
    body.push(Instruction {
        source: instruction.source,
        result: Some(returned),
        operation,
    });
    functions.push(Function {
        id,
        name: target_function.name.clone(),
        signature: FunctionSignature { parameters, result },
        parameters: params,
        locals,
        captures: capture_types
            .iter()
            .map(|&ty| CaptureDeclaration {
                ty,
                kind: CaptureKind::Value,
            })
            .collect(),
        debug,
        blocks: vec![BasicBlock {
            id: BlockId::new(0),
            parameters: vec![],
            instructions: body,
            terminators: vec![Terminator::Return(if result == types::UNIT {
                None
            } else {
                Some(returned.id)
            })],
        }],
        entry: BlockId::new(0),
        max_call_arguments: arguments.len() as u32,
        can_spawn_tasks: target_function.can_spawn_tasks,
    });
    types.errors.displays.insert(
        id,
        if target_function.name.contains('.') {
            target_function.name.clone()
        } else {
            format!("{}.{}", types.errors.owner, target_function.name)
        },
    );
    Ok(Operation::MakeClosure {
        function: id,
        captures: all_captures,
    })
}
