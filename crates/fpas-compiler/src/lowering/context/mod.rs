//! Mutable CFG, lexical-scope, local, and loop lowering state.

mod bindings;
mod block_order;
mod blocks;
mod captures;
mod debug;
mod descriptors;
mod expressions;
mod references;
mod saved_values;

#[cfg(test)]
mod block_order_tests;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{
    BasicBlock, BlockId, BlockTarget, FunctionId, Instruction, Local, LocalId, Operation, TypeId,
    ValueDefinition, ValueId,
};
use fpas_lexer::Span;
use fpas_sema::{ExprTypeMap, Ty};

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::types;

use self::descriptors::{Binding, BindingStorage};

pub(crate) use self::descriptors::{
    BoundMethodTarget, Callable, CaptureInput, ClosureTarget, FunctionInput, GlobalBinding,
    LoopTargets, ParameterInput,
};

pub(super) struct LoweringContext {
    program_name: String,
    pub(super) source_name: String,
    function_id: FunctionId,
    result_type: TypeId,
    parameters: Vec<ValueDefinition>,
    captures: Vec<fpas_ir::CaptureDeclaration>,
    callables: BTreeMap<String, Callable>,
    closure_targets: HashMap<usize, ClosureTarget>,
    pub(super) bound_method_targets: HashMap<usize, BoundMethodTarget>,
    pub(super) intrinsic_task_targets: HashMap<usize, BoundMethodTarget>,
    cell_names: BTreeSet<String>,
    globals: BTreeMap<String, GlobalBinding>,
    constants: BTreeMap<String, fpas_ir::Constant>,
    pub(super) type_table: types::TypeTable,
    pub(super) expr_types: ExprTypeMap,
    /// Full designators resolved to enum members by semantic analysis.
    pub(super) enum_members: std::collections::HashSet<usize>,
    pub(super) intrinsic_calls: fpas_sema::IntrinsicCallMap,
    pub(super) named_argument_orders: fpas_sema::NamedArgumentOrderMap,
    pub(super) record_defaults: fpas_sema::RecordDefaultsMap,
    pub(super) record_constructions: std::collections::HashSet<usize>,
    pub(super) method_calls: fpas_sema::MethodCallMap,
    pub(super) fluent_calls: fpas_sema::FluentCallMap,
    pub(super) member_value_calls: fpas_sema::MemberValueCallMap,
    pub(super) bound_methods: fpas_sema::BoundMethodMap,
    blocks: Vec<BasicBlock>,
    current: BlockId,
    locals: Vec<Local>,
    bindings: Vec<Binding>,
    loops: Vec<LoopTargets>,
    scope_depth: u32,
    debug: fpas_ir::FunctionDebugInfo,
    debug_scope: u32,
    debug_scope_stack: Vec<u32>,
    next_value: u32,
    max_call_arguments: u32,
    pub(super) can_spawn_tasks: bool,
}

impl LoweringContext {
    pub(super) fn new(input: FunctionInput<'_>) -> Result<Self, CompileError> {
        let FunctionInput {
            name,
            source_name,
            id,
            result,
            parameters: parameter_types,
            captures,
            globals,
            constants,
            metadata,
            callables,
            closure_targets,
            bound_method_targets,
            intrinsic_task_targets,
            cell_names,
            type_table,
        } = input;
        let parameters = parameter_types
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                ValueId::try_from_index(index)
                    .map(|id| ValueDefinition {
                        id,
                        ty: parameter.ty,
                    })
                    .map_err(|error| {
                        internal_compiler_error(
                            error.to_string(),
                            "Split the routine into smaller functions.",
                            1,
                            1,
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut locals = Vec::with_capacity(parameter_types.len() + captures.len());
        let mut bindings = Vec::with_capacity(parameter_types.len() + captures.len());
        let mut debug = fpas_ir::FunctionDebugInfo {
            scopes: vec![fpas_ir::DebugScope {
                id: 0,
                parent: None,
            }],
            ..fpas_ir::FunctionDebugInfo::default()
        };
        let mut entry = empty_block(BlockId::new(0));
        for (input, parameter) in parameter_types.iter().zip(&parameters) {
            let local = LocalId::try_from_index(locals.len()).map_err(|error| {
                internal_compiler_error(
                    error.to_string(),
                    "Reduce the number of routine parameters.",
                    1,
                    1,
                )
            })?;
            locals.push(Local {
                id: local,
                ty: input.ty,
                mutable: false,
                capture: None,
            });
            // A `var` parameter binding has the referenced type; its storage is the reference.
            let value_ty = if input.reference {
                referenced_type(&type_table, input.ty)?
            } else {
                input.ty
            };
            debug.bindings.push(fpas_ir::DebugBinding {
                local,
                name: input.name.clone(),
                kind: fpas_ir::DebugBindingKind::Parameter,
                ty: input.ty,
                mutable: false,
                scope: 0,
                declaration: input.declaration,
                hidden: false,
                cell_backed: false,
                initializer: None,
            });
            bindings.push(Binding {
                name: input.name.to_ascii_lowercase(),
                storage: BindingStorage::Local(local),
                ty: value_ty,
                depth: 0,
                declaration: input.declaration,
                cell: false,
                reference: input.reference,
            });
            entry.instructions.push(Instruction {
                source: None,
                result: None,
                operation: Operation::WriteLocal {
                    value: parameter.id,
                    local,
                },
            });
        }
        for (index, capture) in captures.iter().enumerate() {
            let local = LocalId::try_from_index(parameter_types.len().saturating_add(index))
                .map_err(|error| {
                    internal_compiler_error(
                        error.to_string(),
                        "Reduce the number of captured values in this routine.",
                        1,
                        1,
                    )
                })?;
            locals.push(Local {
                id: local,
                ty: capture.storage_ty,
                mutable: capture.kind != fpas_ir::CaptureKind::Value,
                capture: Some(capture.kind),
            });
            debug.bindings.push(fpas_ir::DebugBinding {
                local,
                name: capture.name.clone(),
                kind: fpas_ir::DebugBindingKind::Capture,
                ty: if capture.reference {
                    capture.storage_ty
                } else {
                    capture.ty
                },
                mutable: capture.kind != fpas_ir::CaptureKind::Value,
                scope: 0,
                declaration: capture.declaration,
                hidden: false,
                cell_backed: capture.kind != fpas_ir::CaptureKind::Value,
                initializer: None,
            });
            // Keep parameters visible ahead of same-named transitive captures.
            bindings.insert(
                index,
                Binding {
                    name: capture.name.to_ascii_lowercase(),
                    storage: BindingStorage::Local(local),
                    ty: capture.ty,
                    depth: 0,
                    declaration: capture.declaration,
                    cell: capture.kind != fpas_ir::CaptureKind::Value,
                    reference: capture.reference,
                },
            );
        }
        Ok(Self {
            program_name: name.to_ascii_lowercase(),
            source_name: source_name.to_string(),
            function_id: id,
            result_type: result,
            parameters,
            captures: captures
                .iter()
                .map(|capture| fpas_ir::CaptureDeclaration {
                    // A `var` parameter is captured as its reference value.
                    ty: if capture.reference {
                        capture.storage_ty
                    } else {
                        capture.ty
                    },
                    kind: capture.kind,
                })
                .collect(),
            callables,
            closure_targets,
            bound_method_targets,
            intrinsic_task_targets,
            cell_names,
            globals,
            constants,
            type_table,
            expr_types: metadata.expr_types.clone(),
            enum_members: metadata.enum_members.clone(),
            intrinsic_calls: metadata.intrinsic_calls.clone(),
            named_argument_orders: metadata.named_argument_orders.clone(),
            record_defaults: metadata.record_defaults.clone(),
            record_constructions: metadata.record_constructions.clone(),
            method_calls: metadata.method_calls.clone(),
            fluent_calls: metadata.fluent_calls.clone(),
            member_value_calls: metadata.member_value_calls.clone(),
            bound_methods: metadata.bound_methods.clone(),
            blocks: vec![entry],
            current: BlockId::new(0),
            locals,
            bindings,
            loops: Vec::new(),
            scope_depth: 0,
            debug,
            debug_scope: 0,
            debug_scope_stack: Vec::new(),
            next_value: fpas_ir::checked_count("parameter count", parameter_types.len()).map_err(
                |error| {
                    internal_compiler_error(
                        error.to_string(),
                        "Split the routine into smaller functions.",
                        1,
                        1,
                    )
                },
            )?,
            max_call_arguments: 0,
            can_spawn_tasks: false,
        })
    }

    pub(super) fn specialize_task_binding(&self, declared: TypeId, inferred: TypeId) -> TypeId {
        self.type_table.specialize_task_binding(declared, inferred)
    }

    pub(super) fn is_bare_task_binding(&self, declared: TypeId) -> bool {
        self.type_table.is_bare_task_binding(declared)
    }

    pub(super) fn task_type(
        &mut self,
        inner: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.type_table.intern_task_type(inner, span)
    }

    pub(super) fn function_result_type(&self, callable: TypeId) -> Option<TypeId> {
        self.type_table.function_result(callable)
    }

    pub(super) fn lowered_value_type(&self, value: ValueId) -> Option<TypeId> {
        if let Some(ty) = self
            .parameters
            .iter()
            .find(|definition| definition.id == value)
            .map(|definition| definition.ty)
        {
            return Some(ty);
        }
        if let Some(ty) = self
            .blocks
            .iter()
            .flat_map(|block| &block.parameters)
            .find(|parameter| parameter.id == value)
            .map(|parameter| parameter.ty)
        {
            return Some(ty);
        }
        self.blocks
            .iter()
            .flat_map(|block| &block.instructions)
            .filter_map(|instruction| instruction.result.as_ref())
            .find(|definition| definition.id == value)
            .map(|definition| definition.ty)
    }

    pub(super) fn declared_type(
        &mut self,
        type_expr: &fpas_parser::TypeExpr,
    ) -> Result<TypeId, CompileError> {
        self.type_table.type_expr(type_expr)
    }

    pub(super) fn emit_value(
        &mut self,
        operation: Operation,
        ty: TypeId,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let value = ValueId::new(self.next_value);
        self.next_value = self.next_value.checked_add(1).ok_or_else(|| {
            internal_compiler_error(
                "Register IR value identifier limit exceeded.",
                "Split the program into smaller functions.",
                span.line,
                span.column,
            )
        })?;
        let source = span.diagnostic_span_or_synthetic();
        let instruction = self.current_block_mut()?.instructions.len();
        self.current_block_mut()?.instructions.push(Instruction {
            source: Some(source),
            result: Some(ValueDefinition { id: value, ty }),
            operation,
        });
        self.record_sequence_point(instruction, source);
        Ok(value)
    }

    pub(super) fn emit_effect(
        &mut self,
        operation: Operation,
        span: Span,
    ) -> Result<(), CompileError> {
        self.emit_effect_with_location(operation, span).map(|_| ())
    }

    pub(super) fn emit_effect_with_location(
        &mut self,
        operation: Operation,
        span: Span,
    ) -> Result<fpas_ir::DebugInstructionLocation, CompileError> {
        let source = span.diagnostic_span_or_synthetic();
        let instruction = self.current_block_mut()?.instructions.len();
        self.current_block_mut()?.instructions.push(Instruction {
            source: Some(source),
            result: None,
            operation,
        });
        self.record_sequence_point(instruction, source);
        Ok(fpas_ir::DebugInstructionLocation {
            block: self.current,
            instruction,
        })
    }

    pub(super) fn emit_initializer_store(
        &mut self,
        operation: Operation,
        span: Span,
    ) -> Result<fpas_ir::DebugInstructionLocation, CompileError> {
        let source = span.diagnostic_span_or_synthetic();
        let instruction = self.current_block_mut()?.instructions.len();
        self.current_block_mut()?.instructions.push(Instruction {
            source: Some(source),
            result: None,
            operation,
        });
        Ok(fpas_ir::DebugInstructionLocation {
            block: self.current,
            instruction,
        })
    }
}

pub(super) fn unsupported(span: Span, construct: &str) -> CompileError {
    internal_compiler_error(
        format!("The compiler could not lower `{construct}`."),
        "This is an internal compiler error. Re-run compilation and report the source program.",
        span.line,
        span.column,
    )
}

pub(super) fn target(block: BlockId) -> BlockTarget {
    BlockTarget {
        block,
        arguments: Vec::new(),
    }
}

/// Returns the type behind a `var` parameter's reference type.
fn referenced_type(type_table: &types::TypeTable, ty: TypeId) -> Result<TypeId, CompileError> {
    match type_table.kind(ty) {
        Some(fpas_ir::IrType::Reference(inner)) => Ok(*inner),
        _ => Err(internal_compiler_error(
            "A `var` parameter was lowered without a reference type.",
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        )),
    }
}

fn empty_block(id: BlockId) -> BasicBlock {
    BasicBlock {
        id,
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminators: Vec::new(),
    }
}
