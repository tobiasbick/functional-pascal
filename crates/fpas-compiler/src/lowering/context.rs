//! Mutable CFG, lexical-scope, local, and loop lowering state.

mod bindings;
mod blocks;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{
    BasicBlock, BlockId, BlockTarget, FunctionId, GlobalId, Instruction, Local, LocalId, Operation,
    TypeId, ValueDefinition, ValueId,
};
use fpas_lexer::Span;
use fpas_sema::{AnalysisMetadata, ExprTypeMap, ScalarCaseBindingMap, Ty};

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::types;

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    storage: BindingStorage,
    ty: TypeId,
    depth: u32,
    cell: bool,
}

#[derive(Debug, Clone, Copy)]
enum BindingStorage {
    Local(LocalId),
    Value(ValueId),
}

#[derive(Debug, Clone)]
pub(super) struct Callable {
    pub function: FunctionId,
    pub parameters: Vec<TypeId>,
    pub result: TypeId,
    pub value_type: TypeId,
    pub captures: Vec<CaptureInput>,
}

#[derive(Debug, Clone)]
pub(super) struct CaptureInput {
    pub name: String,
    pub ty: TypeId,
    pub storage_ty: TypeId,
    pub kind: fpas_ir::CaptureKind,
}

#[derive(Debug, Clone)]
pub(super) struct ClosureTarget {
    pub function: FunctionId,
    pub value_type: TypeId,
    pub captures: Vec<CaptureInput>,
}

#[derive(Debug, Clone)]
pub(super) struct BoundMethodTarget {
    pub function: FunctionId,
    pub value_type: TypeId,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LoopTargets {
    pub break_block: BlockId,
    pub continue_block: BlockId,
}

pub(super) struct FunctionInput<'a> {
    pub name: &'a str,
    pub id: FunctionId,
    pub result: TypeId,
    pub parameters: &'a [(String, TypeId)],
    pub captures: &'a [CaptureInput],
    pub globals: BTreeMap<String, GlobalBinding>,
    pub enum_constants: BTreeMap<String, i64>,
    pub metadata: &'a AnalysisMetadata,
    pub callables: BTreeMap<String, Callable>,
    pub closure_targets: HashMap<usize, ClosureTarget>,
    pub bound_method_targets: HashMap<usize, BoundMethodTarget>,
    pub cell_names: BTreeSet<String>,
    pub type_table: types::TypeTable,
}

pub(super) struct LoweringContext {
    program_name: String,
    function_id: FunctionId,
    result_type: TypeId,
    parameters: Vec<ValueDefinition>,
    captures: Vec<fpas_ir::CaptureDeclaration>,
    callables: BTreeMap<String, Callable>,
    closure_targets: HashMap<usize, ClosureTarget>,
    pub(super) bound_method_targets: HashMap<usize, BoundMethodTarget>,
    cell_names: BTreeSet<String>,
    globals: BTreeMap<String, GlobalBinding>,
    enum_constants: BTreeMap<String, i64>,
    type_table: types::TypeTable,
    pub(super) expr_types: ExprTypeMap,
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
    next_value: u32,
    max_call_arguments: u32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GlobalBinding {
    pub id: GlobalId,
    pub ty: TypeId,
}

impl LoweringContext {
    pub(super) fn new(input: FunctionInput<'_>) -> Result<Self, CompileError> {
        let FunctionInput {
            name,
            id,
            result,
            parameters: parameter_types,
            captures,
            globals,
            enum_constants,
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
            .map(|(index, (_, ty))| {
                ValueId::try_from_index(index)
                    .map(|id| ValueDefinition { id, ty: *ty })
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
        let bindings: Vec<Binding> = parameter_types
            .iter()
            .zip(&parameters)
            .map(|((name, ty), parameter)| Binding {
                name: name.to_ascii_lowercase(),
                storage: BindingStorage::Value(parameter.id),
                ty: *ty,
                depth: 0,
                cell: false,
            })
            .collect();
        let mut locals = Vec::with_capacity(captures.len());
        let mut bindings = bindings;
        for (index, capture) in captures.iter().enumerate() {
            let local = LocalId::try_from_index(index).map_err(|error| {
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
            enum_constants,
            type_table,
            expr_types: metadata.expr_types.clone(),
            scalar_case_bindings: metadata.scalar_case_bindings.clone(),
            record_defaults: metadata.record_defaults.clone(),
            method_calls: metadata.method_calls.clone(),
            bound_methods: metadata.bound_methods.clone(),
            property_reads: metadata.property_reads.clone(),
            property_writes: metadata.property_writes.clone(),
            event_writes: metadata.event_writes.clone(),
            event_assigned: metadata.event_assigned.clone(),
            event_raises: metadata.event_raises.clone(),
            blocks: vec![empty_block(BlockId::new(0))],
            current: BlockId::new(0),
            locals,
            bindings,
            loops: Vec::new(),
            scope_depth: 0,
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
                    "Expression type is missing after semantic analysis.",
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
        self.type_table
            .id(&self.expression_type(expression)?, span.line, span.column)
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
        self.current_block_mut()?.instructions.push(Instruction {
            source: Some(span.diagnostic_span_or_synthetic()),
            result: Some(ValueDefinition { id: value, ty }),
            operation,
        });
        Ok(value)
    }

    pub(super) fn emit_effect(
        &mut self,
        operation: Operation,
        span: Span,
    ) -> Result<(), CompileError> {
        self.current_block_mut()?.instructions.push(Instruction {
            source: Some(span.diagnostic_span_or_synthetic()),
            result: None,
            operation,
        });
        Ok(())
    }
}

pub(super) fn unsupported(span: Span, construct: &str) -> CompileError {
    internal_compiler_error(
        format!("`{construct}` is outside the P5 register-development subset."),
        "This development path accepts scalar control flow, routines, closures, globals, and aggregates without imports, intrinsics, tasks, or persistent artifacts.",
        span.line,
        span.column,
    )
}

fn unsupported_parameter_assignment(name: &str, span: Span) -> CompileError {
    internal_compiler_error(
        format!("Assignment to parameter `{name}` is outside the current register subset."),
        "Use a local mutable variable until mutable parameter lowering is added to this development path.",
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

fn reverse_postorder(blocks: &[BasicBlock], entry: BlockId) -> Vec<BasicBlock> {
    fn visit(
        id: BlockId,
        blocks: &[BasicBlock],
        seen: &mut Vec<BlockId>,
        postorder: &mut Vec<BlockId>,
    ) {
        if seen.contains(&id) {
            return;
        }
        seen.push(id);
        if let Some(block) = blocks.iter().find(|block| block.id == id)
            && let Some(terminator) = block.terminators.first()
        {
            for successor in terminator.targets().into_iter().rev() {
                visit(successor.block, blocks, seen, postorder);
            }
        }
        postorder.push(id);
    }

    let mut seen = Vec::new();
    let mut postorder = Vec::new();
    visit(entry, blocks, &mut seen, &mut postorder);
    postorder
        .into_iter()
        .rev()
        .filter_map(|id| blocks.iter().find(|block| block.id == id).cloned())
        .collect()
}
