//! Mutable CFG, lexical-scope, local, and loop lowering state.

mod bindings;
mod block_order;
mod blocks;
mod debug;
mod descriptors;

#[cfg(test)]
mod block_order_tests;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{
    BasicBlock, BlockId, BlockTarget, FunctionId, Instruction, Local, LocalId, Operation, TypeId,
    ValueDefinition, ValueId,
};
use fpas_lexer::Span;
use fpas_sema::{ExprTypeMap, ScalarCaseBindingMap, Ty};

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
    cell_names: BTreeSet<String>,
    globals: BTreeMap<String, GlobalBinding>,
    constants: BTreeMap<String, fpas_ir::Constant>,
    type_table: types::TypeTable,
    pub(super) expr_types: ExprTypeMap,
    pub(super) intrinsic_calls: fpas_sema::IntrinsicCallMap,
    pub(super) scalar_case_bindings: ScalarCaseBindingMap,
    pub(super) record_defaults: fpas_sema::RecordDefaultsMap,
    pub(super) method_calls: fpas_sema::MethodCallMap,
    pub(super) bound_methods: fpas_sema::BoundMethodMap,
    pub(super) property_reads: fpas_sema::PropertyReadMap,
    pub(super) property_writes: fpas_sema::PropertyWriteMap,
    pub(super) event_writes: fpas_sema::EventWriteMap,
    pub(super) event_assigned: fpas_sema::EventAssignedMap,
    pub(super) event_raises: fpas_sema::EventRaiseMap,
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
                mutable: true,
                capture: None,
            });
            debug.bindings.push(fpas_ir::DebugBinding {
                local,
                name: input.name.clone(),
                kind: fpas_ir::DebugBindingKind::Parameter,
                ty: input.ty,
                mutable: true,
                scope: 0,
                declaration: input.declaration,
                hidden: false,
                cell_backed: false,
                initializer: None,
            });
            bindings.push(Binding {
                name: input.name.to_ascii_lowercase(),
                storage: BindingStorage::Local(local),
                ty: input.ty,
                depth: 0,
                cell: false,
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
                ty: capture.ty,
                mutable: capture.kind != fpas_ir::CaptureKind::Value,
                scope: 0,
                declaration: capture.declaration,
                hidden: false,
                cell_backed: capture.kind != fpas_ir::CaptureKind::Value,
                initializer: None,
            });
            bindings.push(Binding {
                name: capture.name.to_ascii_lowercase(),
                storage: BindingStorage::Local(local),
                ty: capture.ty,
                depth: 0,
                cell: capture.kind != fpas_ir::CaptureKind::Value,
            });
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
                    ty: capture.ty,
                    kind: capture.kind,
                })
                .collect(),
            callables,
            closure_targets,
            bound_method_targets,
            cell_names,
            globals,
            constants,
            type_table,
            expr_types: metadata.expr_types.clone(),
            intrinsic_calls: metadata.intrinsic_calls.clone(),
            scalar_case_bindings: metadata.scalar_case_bindings.clone(),
            record_defaults: metadata.record_defaults.clone(),
            method_calls: metadata.method_calls.clone(),
            bound_methods: metadata.bound_methods.clone(),
            property_reads: metadata.property_reads.clone(),
            property_writes: metadata.property_writes.clone(),
            event_writes: metadata.event_writes.clone(),
            event_assigned: metadata.event_assigned.clone(),
            event_raises: metadata.event_raises.clone(),
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

    pub(super) fn expression_type(
        &self,
        expression: &fpas_parser::Expr,
    ) -> Result<Ty, CompileError> {
        self.expr_types
            .get(&fpas_sema::expr_lookup_key(expression))
            .cloned()
            .ok_or_else(|| {
                let span = expression.span();
                internal_compiler_error(
                    format!(
                        "Expression type is missing after semantic analysis for `{expression:?}`."
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })
    }

    pub(super) fn expression_ir_type(
        &self,
        expression: &fpas_parser::Expr,
    ) -> Result<TypeId, CompileError> {
        let span = expression.span();
        if let fpas_parser::Expr::Call { designator, .. } = expression {
            let key = fpas_sema::expr_lookup_key(expression);
            if !self.intrinsic_calls.contains_key(&key) {
                if let Some(result) = self.member_call_result(key) {
                    return Ok(result);
                }
                let qualified = designator
                    .parts
                    .iter()
                    .map(|part| match part {
                        fpas_parser::DesignatorPart::Ident(name, _) => Some(name.as_str()),
                        fpas_parser::DesignatorPart::Index(_, _) => None,
                    })
                    .collect::<Option<Vec<_>>>()
                    .map(|parts| parts.join("."));
                if let Some(result) = qualified
                    .as_deref()
                    .and_then(|name| self.call_result_type(name))
                {
                    return Ok(result);
                }
            }
        }
        if !self
            .expr_types
            .contains_key(&fpas_sema::expr_lookup_key(expression))
            && let fpas_parser::Expr::Designator(designator) = expression
            && let Some(ty) = self.designator_type(designator)
        {
            return Ok(ty);
        }
        self.type_table
            .id(&self.expression_type(expression)?, span.line, span.column)
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

fn empty_block(id: BlockId) -> BasicBlock {
    BasicBlock {
        id,
        parameters: Vec::new(),
        instructions: Vec::new(),
        terminators: Vec::new(),
    }
}
