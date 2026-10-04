//! Array mutation routines share ordinary var-call activation and cleanup.
//!
//! Documentation: `docs/pascal/std/collections/array/mutating.md`.

use std::collections::BTreeMap;

use fpas_ir::{
    BasicBlock, BlockId, Function, FunctionDebugInfo, FunctionId, FunctionSignature, Instruction,
    Local, LocalId, Operation, ReferenceOperation, Terminator, TypeId, ValueDefinition, ValueId,
};
use fpas_lexer::Span;
use fpas_parser::Expr;
use fpas_sema::AnalysisMetadata;

use super::ClosureRegistry;
use crate::CompileError;
use crate::lowering::{
    context::{Callable, unsupported},
    types,
};

/// A concrete array operation invoked through an ordinary var parameter.
pub(in crate::lowering) struct ArrayMutationRoutine {
    id: FunctionId,
    name: String,
    key: String,
    array: TypeId,
    element: TypeId,
    reference: TypeId,
    push: bool,
    span: Span,
}

/// Stable compiler-only lookup name for one resolved native mutation call.
pub(in crate::lowering) fn call_key(push: bool, span: Span) -> String {
    format!(
        "$array_{}_{}_{}",
        if push { "push" } else { "pop" },
        span.source_id,
        span.offset
    )
}

impl ClosureRegistry<'_> {
    /// Register only calls whose semantic symbol is the native array operation.
    pub(super) fn register_array_mutation(
        &mut self,
        key: usize,
        arguments: &[Expr],
        span: Span,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        let Some(name) = metadata.intrinsic_calls.get(&key) else {
            return Ok(());
        };
        let push = name.eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_PUSH);
        if !push && !name.eq_ignore_ascii_case(fpas_std::std_symbols::STD_ARRAY_POP) {
            return Ok(());
        }
        let argument = arguments
            .first()
            .ok_or_else(|| unsupported(span, "array var argument"))?;
        let ty = metadata
            .expr_types
            .get(&fpas_sema::expr_lookup_key(argument))
            .ok_or_else(|| unsupported(span, "array var argument type"))?;
        let fpas_sema::Ty::Array(element) = ty else {
            return Err(unsupported(span, "array var argument type"));
        };
        let array = types.intern(ty, span.line, span.column)?;
        let element = types.intern(element, span.line, span.column)?;
        let reference = types.reference_type(array, span)?;
        let parameters = if push {
            vec![reference, element]
        } else {
            vec![reference]
        };
        let result = if push { types::UNIT } else { element };
        let value_type = types.function_type(parameters.clone(), result, span)?;
        let id = FunctionId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        let key = call_key(push, span);
        self.callables.insert(
            key.clone(),
            Callable {
                function: id,
                parameters,
                result,
                value_type,
                captures: Vec::new(),
            },
        );
        self.array_mutation_routines.push(ArrayMutationRoutine {
            id,
            name: format!("{}.$array_mutation_{}", self.source_name, id.get()),
            key,
            array,
            element,
            reference,
            push,
            span,
        });
        Ok(())
    }

    /// Add discovered array entries to every source lowering context.
    pub(in crate::lowering) fn extend_array_callables(
        &self,
        callables: &mut BTreeMap<String, Callable>,
    ) {
        for routine in &self.array_mutation_routines {
            callables.insert(routine.key.clone(), self.callables[&routine.key].clone());
        }
    }
}

impl ArrayMutationRoutine {
    /// Lower a snapshot update followed by an immediate write through active authority.
    pub(in crate::lowering) fn lower(&self) -> Function {
        let parameter_types = if self.push {
            vec![self.reference, self.element]
        } else {
            vec![self.reference]
        };
        let result = if self.push { types::UNIT } else { self.element };
        let parameters = parameter_types
            .iter()
            .enumerate()
            .map(|(index, ty)| ValueDefinition {
                id: ValueId::new(index as u32),
                ty: *ty,
            })
            .collect::<Vec<_>>();
        let reference = ValueId::new(0);
        let snapshot = ValueId::new(parameters.len() as u32);
        let operation_result = ValueId::new(snapshot.get() + 1);
        let updated = ValueId::new(snapshot.get() + 2);
        let local = LocalId::new(0);
        let instruction = |operation, result| Instruction {
            source: Some(self.span.diagnostic_span_or_synthetic()),
            operation,
            result,
        };
        Function {
            id: self.id,
            name: self.name.clone(),
            signature: FunctionSignature {
                parameters: parameter_types,
                result,
            },
            parameters,
            locals: vec![Local {
                id: local,
                ty: self.array,
                mutable: true,
                capture: None,
            }],
            captures: Vec::new(),
            debug: FunctionDebugInfo::default(),
            blocks: vec![BasicBlock {
                id: BlockId::new(0),
                parameters: Vec::new(),
                instructions: vec![
                    instruction(
                        Operation::Reference(ReferenceOperation::Read(reference)),
                        Some(ValueDefinition {
                            id: snapshot,
                            ty: self.array,
                        }),
                    ),
                    instruction(
                        Operation::WriteLocal {
                            local,
                            value: snapshot,
                        },
                        None,
                    ),
                    instruction(
                        if self.push {
                            Operation::ArrayPush {
                                local,
                                value: ValueId::new(1),
                            }
                        } else {
                            Operation::ArrayPop { local }
                        },
                        Some(ValueDefinition {
                            id: operation_result,
                            ty: result,
                        }),
                    ),
                    instruction(
                        Operation::ReadLocal(local),
                        Some(ValueDefinition {
                            id: updated,
                            ty: self.array,
                        }),
                    ),
                    instruction(
                        Operation::Reference(ReferenceOperation::Write {
                            reference,
                            value: updated,
                        }),
                        None,
                    ),
                ],
                terminators: vec![Terminator::Return(if self.push {
                    None
                } else {
                    Some(operation_result)
                })],
            }],
            entry: BlockId::new(0),
            max_call_arguments: 0,
            can_spawn_tasks: false,
        }
    }
}
