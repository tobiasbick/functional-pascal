//! Mutable CFG, lexical-scope, local, and loop lowering state.

mod bindings;
mod block_order;
mod blocks;
mod debug;
mod descriptors;
mod designators;
mod globals;
mod imports;
mod initialization;
mod saved_values;

#[cfg(test)]
mod block_order_tests;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{
    BasicBlock, BlockId, BlockTarget, FunctionId, Instruction, Local, Operation, TypeId,
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
    pub(super) intrinsic_task_targets: HashMap<usize, BoundMethodTarget>,
    cell_names: BTreeSet<String>,
    globals: BTreeMap<String, GlobalBinding>,
    constants: BTreeMap<String, fpas_ir::Constant>,
    import_aliases: HashMap<String, String>,
    pub(super) type_table: types::TypeTable,
    pub(super) expr_types: ExprTypeMap,
    pub(super) intrinsic_calls: fpas_sema::IntrinsicCallMap,
    pub(super) scalar_case_bindings: ScalarCaseBindingMap,
    pub(super) record_defaults: fpas_sema::RecordDefaultsMap,
    pub(super) method_calls: fpas_sema::MethodCallMap,
    pub(super) fluent_calls: fpas_sema::FluentCallMap,
    pub(super) value_calls: fpas_sema::ValueCallMap,
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
