use super::super::super::Compiler;
use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, Op, SourceLocation, Value};
use fpas_parser::{Expr, ForDirection, Stmt};

impl Compiler {
    pub(in super::super) fn compile_for_stmt(
        &mut self,
        var_name: &str,
        start: &Expr,
        direction: &ForDirection,
        end: &Expr,
        body: &Stmt,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.compile_expr(start)?;
        self.begin_scope();
        let var_slot = self.add_local(var_name);

        self.compile_expr(end)?;
        let end_slot = self.add_local("__for_end");

        let loop_start = self.chunk.len();
        self.push_loop_context();

        self.emit(Op::GetLocal(var_slot), location);
        self.emit(Op::GetLocal(end_slot), location);
        match direction {
            ForDirection::To => self.emit(Op::LeInt, location),
            ForDirection::Downto => self.emit(Op::GeInt, location),
        };
        let exit_jump = self.emit(Op::JumpIfFalse(0), location);

        self.compile_stmt(body)?;

        let increment_start = self.chunk.len() as u32;
        self.patch_continues(increment_start, location)?;

        self.emit(Op::GetLocal(var_slot), location);
        self.emit_constant(Value::Integer(1), location)?;
        match direction {
            ForDirection::To => self.emit(Op::AddInt, location),
            ForDirection::Downto => self.emit(Op::SubInt, location),
        };
        self.emit(Op::SetLocal(var_slot), location);
        self.emit(Op::Pop, location);

        self.emit(Op::Jump(loop_start as u32), location);

        let after = self.chunk.len() as u32;
        self.patch_jump(exit_jump, after, location)?;
        self.patch_and_pop_breaks(after, location)?;
        self.end_scope(location);
        Ok(())
    }

    pub(in super::super) fn compile_for_in_stmt(
        &mut self,
        var_name: &str,
        iterable: &Expr,
        body: &Stmt,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        use fpas_sema::Ty;
        self.compile_expr(iterable)?;
        // For dict iterables, convert to array of keys first so the rest of the
        // loop body uses the identical array-iteration pattern.
        if matches!(self.ty_of(iterable), Ty::Dict(_, _)) {
            self.emit(Op::Intrinsic(u16::from(Intrinsic::DictKeys)), location);
        }
        self.begin_scope();
        let arr_slot = self.add_local("__for_arr");

        self.emit_constant(Value::Integer(0), location)?;
        let idx_slot = self.add_local("__for_idx");

        self.emit(Op::GetLocal(arr_slot), location);
        self.emit(Op::Intrinsic(u16::from(Intrinsic::ArrayLength)), location);
        let len_slot = self.add_local("__for_len");

        self.emit(Op::Unit, location);
        let var_slot = self.add_local(var_name);

        let loop_start = self.chunk.len();
        self.push_loop_context();

        self.emit(Op::GetLocal(idx_slot), location);
        self.emit(Op::GetLocal(len_slot), location);
        self.emit(Op::LtInt, location);
        let exit_jump = self.emit(Op::JumpIfFalse(0), location);

        self.emit(Op::GetLocal(arr_slot), location);
        self.emit(Op::GetLocal(idx_slot), location);
        self.emit(Op::IndexGet, location);
        self.emit(Op::SetLocal(var_slot), location);
        self.emit(Op::Pop, location);

        self.compile_stmt(body)?;

        let increment_start = self.chunk.len() as u32;
        self.patch_continues(increment_start, location)?;

        self.emit(Op::GetLocal(idx_slot), location);
        self.emit_constant(Value::Integer(1), location)?;
        self.emit(Op::AddInt, location);
        self.emit(Op::SetLocal(idx_slot), location);
        self.emit(Op::Pop, location);

        self.emit(Op::Jump(loop_start as u32), location);

        let after = self.chunk.len() as u32;
        self.patch_jump(exit_jump, after, location)?;
        self.patch_and_pop_breaks(after, location)?;
        self.end_scope(location);
        Ok(())
    }
}
