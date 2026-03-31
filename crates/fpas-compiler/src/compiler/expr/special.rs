use crate::error::CompileError;
use fpas_bytecode::{Op, Value};
use fpas_parser::{Expr, FormalParam, FuncBody};

use super::super::Compiler;

impl Compiler {
    pub(super) fn compile_try_expr(
        &mut self,
        inner: &Expr,
        location: (u32, u32),
    ) -> Result<(), CompileError> {
        self.compile_expr(inner)?;
        self.emit(Op::Dup, location);
        self.emit(Op::IsResultOk, location);
        let jump_ok = self.chunk.code.len();
        self.emit(Op::JumpIfTrue(0), location);
        self.emit(Op::Return, location);

        let ok_addr = self.chunk.code.len() as u32;
        self.patch_jump(jump_ok, ok_addr, location)?;
        self.emit(Op::UnwrapOk, location);
        Ok(())
    }

    /// Compile a compiler-generated callable (used internally for `go` wrappers).
    ///
    /// Emits a jump over the body, compiles the body as a named routine, registers it
    /// in the chunk, then patches the jump and pushes the resulting `Value::Function`
    /// on the stack. No user-visible syntax is involved.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    pub(in crate::compiler) fn compile_callable_wrapper(
        &mut self,
        params: &[FormalParam],
        body: &FuncBody,
        location: (u32, u32),
    ) -> Result<(), CompileError> {
        let wrapper_name = format!("$wrapper_{}", self.chunk.code.len());
        let arity = params.len() as u8;

        let jump_over = self.emit(Op::Jump(0), location);
        let (code_start, _) = self.compile_routine_body(params, body, location)?;
        self.chunk
            .functions
            .insert(wrapper_name.clone(), (code_start, arity));

        let after = self.chunk.len() as u32;
        self.patch_jump(jump_over, after, location)?;

        self.emit_constant(
            Value::Function {
                name: wrapper_name,
                captures: vec![],
            },
            location,
        );
        Ok(())
    }
}
