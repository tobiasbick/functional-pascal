//! Mutable CFG, lexical-scope, local, and loop lowering state.

use fpas_ir::{
    BasicBlock, BlockId, BlockTarget, Function, FunctionId, FunctionSignature, Instruction, Local,
    LocalId, Operation, Terminator, TypeId, ValueDefinition, ValueId,
};
use fpas_lexer::Span;
use fpas_sema::{AnalysisMetadata, ExprTypeMap, ScalarCaseBindingMap, Ty};

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::types;

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    local: LocalId,
    ty: TypeId,
    depth: u32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LoopTargets {
    pub break_block: BlockId,
    pub continue_block: BlockId,
}

pub(super) struct LoweringContext {
    program_name: String,
    pub(super) expr_types: ExprTypeMap,
    pub(super) scalar_case_bindings: ScalarCaseBindingMap,
    blocks: Vec<BasicBlock>,
    current: BlockId,
    locals: Vec<Local>,
    bindings: Vec<Binding>,
    loops: Vec<LoopTargets>,
    scope_depth: u32,
    next_value: u32,
}

impl LoweringContext {
    pub(super) fn new(program_name: &str, metadata: AnalysisMetadata) -> Self {
        Self {
            program_name: program_name.to_ascii_lowercase(),
            expr_types: metadata.expr_types,
            scalar_case_bindings: metadata.scalar_case_bindings,
            blocks: vec![empty_block(BlockId::new(0))],
            current: BlockId::new(0),
            locals: Vec::new(),
            bindings: Vec::new(),
            loops: Vec::new(),
            scope_depth: 0,
            next_value: 0,
        }
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
        types::lower(&self.expression_type(expression)?, span.line, span.column)
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

    pub(super) fn declare_local(
        &mut self,
        name: &str,
        ty: TypeId,
        mutable: bool,
        span: Span,
    ) -> Result<LocalId, CompileError> {
        let local = LocalId::try_from_index(self.locals.len()).map_err(|error| {
            internal_compiler_error(
                error.to_string(),
                "Split the program into smaller functions.",
                span.line,
                span.column,
            )
        })?;
        self.locals.push(Local {
            id: local,
            ty,
            mutable,
            capture: None,
        });
        self.bindings.push(Binding {
            name: name.to_ascii_lowercase(),
            local,
            ty,
            depth: self.scope_depth,
        });
        Ok(local)
    }

    pub(super) fn declare_hidden_local(
        &mut self,
        ty: TypeId,
        span: Span,
    ) -> Result<LocalId, CompileError> {
        let name = format!("$p3_{}", self.locals.len());
        self.declare_local(&name, ty, true, span)
    }

    pub(super) fn resolve_local(
        &self,
        name: &str,
        span: Span,
    ) -> Result<(LocalId, TypeId), CompileError> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name.eq_ignore_ascii_case(name))
            .map(|binding| (binding.local, binding.ty))
            .ok_or_else(|| {
                internal_compiler_error(
                    format!("Local `{name}` was not present in register-lowering scope metadata."),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                )
            })
    }

    pub(super) fn read_named_local(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<ValueId, CompileError> {
        let (local, ty) = self.resolve_local(name, span)?;
        self.emit_value(Operation::ReadLocal(local), ty, span)
    }

    pub(super) fn write_local(
        &mut self,
        local: LocalId,
        value: ValueId,
        span: Span,
    ) -> Result<(), CompileError> {
        self.emit_effect(Operation::WriteLocal { value, local }, span)
    }

    pub(super) fn begin_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_add(1);
    }

    pub(super) fn end_scope(&mut self) {
        let depth = self.scope_depth;
        self.bindings.retain(|binding| binding.depth < depth);
        self.scope_depth = self.scope_depth.saturating_sub(1);
    }

    pub(super) fn new_block(&mut self, span: Span) -> Result<BlockId, CompileError> {
        let id = BlockId::try_from_index(self.blocks.len()).map_err(|error| {
            internal_compiler_error(
                error.to_string(),
                "Split the program into smaller functions.",
                span.line,
                span.column,
            )
        })?;
        self.blocks.push(empty_block(id));
        Ok(id)
    }

    pub(super) fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    pub(super) fn is_terminated(&self) -> bool {
        self.block(self.current)
            .is_none_or(|block| !block.terminators.is_empty())
    }

    pub(super) fn terminate(&mut self, terminator: Terminator) -> Result<(), CompileError> {
        let current = self.current;
        let block = self.current_block_mut()?;
        if !block.terminators.is_empty() {
            return Err(internal_compiler_error(
                format!(
                    "Register IR block {} received multiple terminators.",
                    current.get()
                ),
                "This is an internal compiler error. Re-run compilation and report the source program.",
                1,
                1,
            ));
        }
        block.terminators.push(terminator);
        Ok(())
    }

    pub(super) fn set_last_instruction_source(&mut self, source: Span) -> Result<(), CompileError> {
        let block = self.current_block_mut()?;
        let Some(instruction) = block.instructions.last_mut() else {
            return Err(internal_compiler_error(
                "A source-bearing terminator has no preceding value instruction.",
                "This is an internal compiler error. Re-run compilation and report the source program.",
                source.line,
                source.column,
            ));
        };
        instruction.source = Some(source.diagnostic_span_or_synthetic());
        Ok(())
    }

    pub(super) fn jump(&mut self, block: BlockId) -> Result<(), CompileError> {
        self.terminate(Terminator::Jump(target(block)))
    }

    pub(super) fn push_loop(&mut self, targets: LoopTargets) {
        self.loops.push(targets);
    }

    pub(super) fn pop_loop(&mut self) {
        let _ = self.loops.pop();
    }

    pub(super) fn loop_targets(&self, span: Span) -> Result<LoopTargets, CompileError> {
        self.loops.last().copied().ok_or_else(|| {
            internal_compiler_error(
                "Loop control reached lowering without an active loop.",
                "Use `break` or `continue` only inside a loop.",
                span.line,
                span.column,
            )
        })
    }

    pub(super) fn remove_last_block_if(&mut self, id: BlockId) {
        if self.blocks.last().is_some_and(|block| block.id == id) {
            let _ = self.blocks.pop();
        }
    }

    pub(super) fn finish(mut self, span: Span) -> Result<Function, CompileError> {
        if !self.is_terminated() {
            self.terminate(Terminator::Return(None))?;
        }
        let blocks = reverse_postorder(&self.blocks, BlockId::new(0));
        if blocks.is_empty() {
            return Err(internal_compiler_error(
                "Register IR root has no reachable entry block.",
                "This is an internal compiler error. Re-run compilation and report the source program.",
                span.line,
                span.column,
            ));
        }
        Ok(Function {
            id: FunctionId::new(0),
            name: self.program_name,
            signature: FunctionSignature {
                parameters: Vec::new(),
                result: types::UNIT,
            },
            parameters: Vec::new(),
            locals: self.locals,
            captures: Vec::new(),
            blocks,
            entry: BlockId::new(0),
            max_call_arguments: 0,
            can_spawn_tasks: false,
        })
    }

    fn current_block_mut(&mut self) -> Result<&mut BasicBlock, CompileError> {
        let id = self.current;
        self.blocks
            .iter_mut()
            .find(|block| block.id == id)
            .ok_or_else(|| {
                internal_compiler_error(
                    format!("Register IR current block {} is missing.", id.get()),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    1,
                    1,
                )
            })
    }

    fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|block| block.id == id)
    }
}

pub(super) fn unsupported(span: Span, construct: &str) -> CompileError {
    internal_compiler_error(
        format!("`{construct}` is outside the P3 scalar/control-flow register subset."),
        "This development path currently accepts functionless scalar programs without imports, globals, aggregates, calls, intrinsics, or tasks.",
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
