//! Statement lowering for `while`, `repeat`, `for`, `for-in`, `break`, and `continue`.
//!
//! **Documentation:** `docs/pascal/03-control-flow.md` (from the repository root).

mod control;
mod for_loops;
mod while_repeat;

use super::super::{Compiler, LoopCtx};
use crate::compiler::emit::IntoEmitLocation;
use crate::error::CompileError;

impl Compiler {
    fn push_loop_context(&mut self) {
        self.loop_stack.push(LoopCtx {
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
        });
    }

    fn emit_loop_scope_pops(&mut self, location: impl IntoEmitLocation + Copy) {
        if let Some(ctx) = self.loop_stack.last() {
            let pops = self
                .locals
                .iter()
                .rev()
                .take_while(|local| local.depth > ctx.scope_depth)
                .count();
            for _ in 0..pops {
                self.emit(fpas_bytecode::Op::Pop, location);
            }
        }
    }

    fn patch_continues(
        &mut self,
        target: u32,
        location: impl IntoEmitLocation + Copy,
    ) -> Result<(), CompileError> {
        let patches = self
            .loop_stack
            .last_mut()
            .map(|ctx| std::mem::take(&mut ctx.continue_patches))
            .unwrap_or_default();

        for patch in patches {
            self.patch_jump(patch, target, location)?;
        }
        Ok(())
    }

    fn patch_and_pop_breaks(
        &mut self,
        after: u32,
        location: impl IntoEmitLocation + Copy,
    ) -> Result<(), CompileError> {
        if let Some(ctx) = self.loop_stack.pop() {
            for patch in ctx.break_patches {
                self.patch_jump(patch, after, location)?;
            }
        }
        Ok(())
    }
}
