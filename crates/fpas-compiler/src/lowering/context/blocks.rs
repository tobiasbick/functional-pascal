//! Basic-block, terminator, loop-target, and CFG finalization state.

use fpas_ir::{BasicBlock, BlockId, Function, FunctionSignature, Terminator};
use fpas_lexer::Span;

use crate::CompileError;
use crate::error::internal_compiler_error;

use super::{LoopTargets, LoweringContext, reverse_postorder, target};

impl LoweringContext {
    pub(in crate::lowering) fn new_block(&mut self, span: Span) -> Result<BlockId, CompileError> {
        let id = BlockId::try_from_index(self.blocks.len()).map_err(|error| {
            internal_compiler_error(
                error.to_string(),
                "Split the program into smaller functions.",
                span.line,
                span.column,
            )
        })?;
        self.blocks.push(super::empty_block(id));
        Ok(id)
    }

    pub(in crate::lowering) fn switch_to(&mut self, block: BlockId) {
        self.current = block;
    }

    pub(in crate::lowering) fn is_terminated(&self) -> bool {
        self.block(self.current)
            .is_none_or(|block| !block.terminators.is_empty())
    }

    pub(in crate::lowering) fn terminate(
        &mut self,
        terminator: Terminator,
    ) -> Result<(), CompileError> {
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

    pub(in crate::lowering) fn set_last_instruction_source(
        &mut self,
        source: Span,
    ) -> Result<(), CompileError> {
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

    pub(in crate::lowering) fn jump(&mut self, block: BlockId) -> Result<(), CompileError> {
        self.terminate(Terminator::Jump(target(block)))
    }

    pub(in crate::lowering) fn push_loop(&mut self, targets: LoopTargets) {
        self.loops.push(targets);
    }

    pub(in crate::lowering) fn pop_loop(&mut self) {
        let _ = self.loops.pop();
    }

    pub(in crate::lowering) fn loop_targets(
        &self,
        span: Span,
    ) -> Result<LoopTargets, CompileError> {
        self.loops.last().copied().ok_or_else(|| {
            internal_compiler_error(
                "Loop control reached lowering without an active loop.",
                "Use `break` or `continue` only inside a loop.",
                span.line,
                span.column,
            )
        })
    }

    pub(in crate::lowering) fn remove_last_block_if(&mut self, id: BlockId) {
        if self.blocks.last().is_some_and(|block| block.id == id) {
            let _ = self.blocks.pop();
        }
    }

    /// Finalizes reachable code and its source sequence points into one IR function.
    pub(in crate::lowering) fn finish(
        mut self,
        span: Span,
    ) -> Result<(Function, super::types::TypeTable), CompileError> {
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
        let reachable: std::collections::BTreeSet<_> =
            blocks.iter().map(|block| block.id).collect();
        self.debug
            .sequence_points
            .retain(|point| reachable.contains(&point.block));
        let function = Function {
            id: self.function_id,
            name: self.program_name,
            signature: FunctionSignature {
                parameters: self
                    .parameters
                    .iter()
                    .map(|parameter| parameter.ty)
                    .collect(),
                result: self.result_type,
            },
            parameters: self.parameters,
            locals: self.locals,
            captures: self.captures,
            debug: self.debug,
            blocks,
            entry: BlockId::new(0),
            max_call_arguments: self.max_call_arguments,
            can_spawn_tasks: self.can_spawn_tasks,
        };
        Ok((function, self.type_table))
    }

    pub(super) fn current_block_mut(&mut self) -> Result<&mut BasicBlock, CompileError> {
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
