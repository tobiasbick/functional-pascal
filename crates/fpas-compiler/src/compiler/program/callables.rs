//! Compiling named functions and procedures.
//!
//! **Documentation:** `docs/pascal/04-functions.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::Op;
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl, RecordMethod};

use super::Compiler;

impl Compiler {
    pub(super) fn compile_function(&mut self, function: &FunctionDecl) -> Result<(), CompileError> {
        self.compile_callable(
            &function.name,
            &function.params,
            &function.body,
            function.span,
        )
    }

    pub(super) fn compile_procedure(
        &mut self,
        procedure: &ProcedureDecl,
    ) -> Result<(), CompileError> {
        self.compile_callable(
            &procedure.name,
            &procedure.params,
            &procedure.body,
            procedure.span,
        )
    }

    pub(super) fn compile_record_method(
        &mut self,
        type_name: &str,
        method: &RecordMethod,
    ) -> Result<(), CompileError> {
        match method {
            RecordMethod::Function(function) => {
                let qualified = format!("{type_name}.{}", function.name);
                self.compile_callable(&qualified, &function.params, &function.body, function.span)
            }
            RecordMethod::Procedure(procedure) => {
                let qualified = format!("{type_name}.{}", procedure.name);
                self.compile_callable(
                    &qualified,
                    &procedure.params,
                    &procedure.body,
                    procedure.span,
                )
            }
        }
    }

    fn compile_callable(
        &mut self,
        name: &str,
        params: &[fpas_parser::FormalParam],
        body: &FuncBody,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        if matches!(body, FuncBody::SignatureOnly) {
            return Ok(());
        }

        let location = Self::location_of(&span);
        let jump_over = self.emit(Op::Jump(0), location);

        let code_start = self.chunk.len();
        self.chunk
            .functions
            .insert(name.to_string(), (code_start, params.len() as u8));

        self.compile_routine_body(params, body, location)?;

        let after = self.chunk.len() as u32;
        self.patch_jump(jump_over, after, location)?;
        Ok(())
    }

    /// Shared body compilation for named callables and anonymous lambdas.
    ///
    /// Saves and restores the compiler's local-variable state, compiles the
    /// function body inside a fresh scope, and emits the trailing `Unit`+`Return`.
    /// Returns `(code_start, body_end)` byte offsets for the caller.
    pub(in crate::compiler) fn compile_routine_body(
        &mut self,
        params: &[fpas_parser::FormalParam],
        body: &FuncBody,
        location: impl super::super::emit::IntoEmitLocation + Copy,
    ) -> Result<(usize, usize), CompileError> {
        let code_start = self.chunk.len();

        let saved_locals = std::mem::take(&mut self.locals);
        let saved_next_slot = self.next_slot;
        let saved_scope_depth = self.scope_depth;
        self.next_slot = 0;
        self.scope_depth = 0;
        self.enclosing_locals.push(saved_locals.clone());

        self.begin_scope();

        for param in params {
            self.add_local(&param.name);
        }

        if let FuncBody::Block { nested, stmts } = body {
            for decl in nested {
                self.compile_decl(decl)?;
            }
            for stmt in stmts {
                self.compile_stmt(stmt)?;
            }
        }

        self.emit(Op::Unit, location);
        self.emit(Op::Return, location);
        self.end_scope(location);

        let body_end = self.chunk.len();

        self.enclosing_locals.pop();
        self.locals = saved_locals;
        self.next_slot = saved_next_slot;
        self.scope_depth = saved_scope_depth;

        Ok((code_start, body_end))
    }
}
