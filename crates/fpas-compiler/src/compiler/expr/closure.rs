//! Lowering for anonymous function / procedure expressions (closures).
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, Value};
use fpas_parser::{Expr, FormalParam, FuncBody, TypeExpr};

use super::super::Compiler;

impl Compiler {
    /// Compile a closure expression as a nested function with its capture environment.
    ///
    /// Emits a jump over the body, registers the synthetic routine, then
    /// [`Op::MakeClosure`] with values or cells from semantic capture analysis.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(super) fn compile_closure_expr(
        &mut self,
        expr: &Expr,
        params: &[FormalParam],
        _return_type: &Option<TypeExpr>,
        body: &FuncBody,
        span: fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let location = Self::location_of(&span);
        let closure_info = self
            .closure_infos
            .get(&fpas_sema::expr_lookup_key(expr))
            .cloned()
            .ok_or_else(|| {
                internal_compiler_error(
                    "Missing semantic capture metadata for closure expression.",
                    "Re-run compilation and report this internal compiler error.",
                    span.line,
                    span.column,
                )
            })?;
        let closure_name = closure_info.synthetic_name.clone();
        let arity = Self::checked_u8(params.len(), "closure parameters", span)?;

        let jump_over = self.emit(Op::Jump(0), location);
        let (code_start, _) = self.compile_routine_body_with_captures(
            params,
            Some(&closure_info.captures),
            body,
            location,
        )?;
        self.chunk
            .insert_function(closure_name.clone(), code_start, arity);

        let after = self.chunk.len() as u32;
        self.patch_jump(jump_over, after, location)?;

        for capture in &closure_info.captures {
            self.emit_load_capture(capture, location);
        }
        let name_idx = self.add_constant(Value::Str(closure_name.into()), location)?;
        let capture_count =
            Self::checked_u8_at(closure_info.captures.len(), "closure captures", location)?;
        self.emit(Op::MakeClosure(name_idx, capture_count), location);
        Ok(())
    }
}
