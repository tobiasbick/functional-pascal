//! Statement lowering for `go` (task spawning) and `select` (channel multiplexing).
//!
//! **Documentation:** `docs/pascal/08-concurrency.md`

use super::super::Compiler;
use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Intrinsic, Op, Value};
use fpas_diagnostics::codes::COMPILE_INVALID_GO_EXPRESSION;
use fpas_lexer::Span;
use fpas_parser::{Expr, SelectArm, Stmt};

impl Compiler {
    /// Compile `go Func(args)` as a statement (fire-and-forget: discard task handle).
    pub(super) fn compile_go_stmt(&mut self, expr: &Expr, span: Span) -> Result<(), CompileError> {
        self.compile_go_expr(expr, span)?;
        self.emit(Op::Pop, (span.line, span.column));
        Ok(())
    }

    /// Compile `go CallExpr` as an expression that pushes a `Value::Task(id)`.
    pub(crate) fn compile_go_expr(&mut self, expr: &Expr, span: Span) -> Result<(), CompileError> {
        let loc = (span.line, span.column);
        match expr {
            Expr::Call {
                designator, args, ..
            } => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name = Self::resolve_designator_name(designator);
                let idx = self.chunk.add_constant(Value::Function {
                    name,
                    captures: vec![],
                });
                self.emit(Op::Constant(idx), loc);
                self.emit(Op::SpawnTask(args.len() as u8), loc);
                Ok(())
            }
            _ => Err(compile_error(
                COMPILE_INVALID_GO_EXPRESSION,
                "`go` requires a function call",
                "Use `go FunctionName(args)` to spawn a task.",
                span,
            )),
        }
    }

    /// Compile `select ... end`.
    pub(super) fn compile_select_stmt(
        &mut self,
        arms: &[SelectArm],
        default_body: Option<&[Stmt]>,
        span: Span,
    ) -> Result<(), CompileError> {
        let loc = (span.line, span.column);
        let loop_start = self.chunk.len();
        let mut skip_jumps: Vec<usize> = Vec::new();

        for arm in arms {
            self.compile_expr(&arm.channel)?;
            self.emit(Op::Intrinsic(Intrinsic::ChannelTryRecv as u16), loc);

            self.emit(Op::Dup, loc);
            self.emit(Op::IsOptionSome, loc);
            let skip_arm = self.emit(Op::JumpIfFalse(0), loc);

            self.emit(Op::UnwrapSome, loc);
            self.begin_scope();
            self.add_local(&arm.binding);
            self.compile_stmt(&arm.body)?;
            self.end_scope(loc);
            skip_jumps.push(self.emit(Op::Jump(0), loc));

            let after_arm = self.chunk.len() as u32;
            self.patch_jump(skip_arm, after_arm, loc)?;
            self.emit(Op::Pop, loc);
        }

        if let Some(body) = default_body {
            for stmt in body {
                self.compile_stmt(stmt)?;
            }
        } else if !arms.is_empty() {
            self.emit(Op::Yield, loc);
            self.emit(Op::Jump(loop_start as u32), loc);
        }

        let select_end = self.chunk.len() as u32;
        for jump in skip_jumps {
            self.patch_jump(jump, select_end, loc)?;
        }

        Ok(())
    }
}
