use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, Op, Value};
use fpas_parser::{Expr, ForDirection, Stmt};

impl Compiler {
    pub(in super::super) fn compile_for_stmt(
        &mut self,
        var_name: &str,
        start: &Expr,
        direction: &ForDirection,
        end: &Expr,
        body: &Stmt,
        location: (u32, u32),
    ) -> Result<(), CompileError> {
        let (line, column) = location;
        self.compile_expr(start)?;
        self.begin_scope();
        let var_slot = self.add_local(var_name);

        self.compile_expr(end)?;
        let end_slot = self.add_local("__for_end");

        let loop_start = self.chunk.len();
        self.push_loop_context();

        self.emit(Op::GetLocal(var_slot), (line, column));
        self.emit(Op::GetLocal(end_slot), (line, column));
        match direction {
            ForDirection::To => self.emit(Op::LeInt, (line, column)),
            ForDirection::Downto => self.emit(Op::GeInt, (line, column)),
        };
        let exit_jump = self.emit(Op::JumpIfFalse(0), (line, column));

        self.compile_stmt(body)?;

        let increment_start = self.chunk.len() as u32;
        self.patch_continues(increment_start, (line, column))?;

        self.emit(Op::GetLocal(var_slot), (line, column));
        self.emit_constant(Value::Integer(1), (line, column))?;
        match direction {
            ForDirection::To => self.emit(Op::AddInt, (line, column)),
            ForDirection::Downto => self.emit(Op::SubInt, (line, column)),
        };
        self.emit(Op::SetLocal(var_slot), (line, column));
        self.emit(Op::Pop, (line, column));

        self.emit(Op::Jump(loop_start as u32), (line, column));

        let after = self.chunk.len() as u32;
        self.patch_jump(exit_jump, after, (line, column))?;
        self.patch_and_pop_breaks(after, (line, column))?;
        self.end_scope((line, column));
        Ok(())
    }

    pub(in super::super) fn compile_for_in_stmt(
        &mut self,
        var_name: &str,
        iterable: &Expr,
        body: &Stmt,
        line: u32,
        column: u32,
    ) -> Result<(), CompileError> {
        use fpas_sema::Ty;
        self.compile_expr(iterable)?;
        // For dict iterables, convert to array of keys first so the rest of the
        // loop body uses the identical array-iteration pattern.
        if matches!(self.ty_of(iterable), Ty::Dict(_, _)) {
            self.emit(
                Op::Intrinsic(u16::from(Intrinsic::DictKeys)),
                (line, column),
            );
        }
        self.begin_scope();
        let arr_slot = self.add_local("__for_arr");

        self.emit_constant(Value::Integer(0), (line, column))?;
        let idx_slot = self.add_local("__for_idx");

        self.emit(Op::GetLocal(arr_slot), (line, column));
        self.emit(
            Op::Intrinsic(u16::from(Intrinsic::ArrayLength)),
            (line, column),
        );
        let len_slot = self.add_local("__for_len");

        self.emit(Op::Unit, (line, column));
        let var_slot = self.add_local(var_name);

        let loop_start = self.chunk.len();
        self.push_loop_context();

        self.emit(Op::GetLocal(idx_slot), (line, column));
        self.emit(Op::GetLocal(len_slot), (line, column));
        self.emit(Op::LtInt, (line, column));
        let exit_jump = self.emit(Op::JumpIfFalse(0), (line, column));

        self.emit(Op::GetLocal(arr_slot), (line, column));
        self.emit(Op::GetLocal(idx_slot), (line, column));
        self.emit(Op::IndexGet, (line, column));
        self.emit(Op::SetLocal(var_slot), (line, column));
        self.emit(Op::Pop, (line, column));

        self.compile_stmt(body)?;

        let increment_start = self.chunk.len() as u32;
        self.patch_continues(increment_start, (line, column))?;

        self.emit(Op::GetLocal(idx_slot), (line, column));
        self.emit_constant(Value::Integer(1), (line, column))?;
        self.emit(Op::AddInt, (line, column));
        self.emit(Op::SetLocal(idx_slot), (line, column));
        self.emit(Op::Pop, (line, column));

        self.emit(Op::Jump(loop_start as u32), (line, column));

        let after = self.chunk.len() as u32;
        self.patch_jump(exit_jump, after, (line, column))?;
        self.patch_and_pop_breaks(after, (line, column))?;
        self.end_scope((line, column));
        Ok(())
    }
}
