//! Lowering for [`Expr::Postfix`] chains.
//!
//! Compiles the base once, then applies each suffix as `FieldGet` / `IndexGet` / instance
//! `Call`. Method targets come from [`fpas_sema::MethodCallMap`] keyed by
//! [`fpas_sema::postfix_operation_lookup_key`].
//!
//! **Documentation:** `docs/pascal/language/functions/README.md`

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, Value};
use fpas_parser::{Expr, PostfixOperation};

use super::super::Compiler;

impl Compiler {
    /// Lower a postfix expression: compile `base`, then each field / index / method suffix.
    pub(super) fn compile_postfix_expr(
        &mut self,
        base: &Expr,
        operations: &[PostfixOperation],
    ) -> Result<(), CompileError> {
        self.compile_expr(base)?;
        for op in operations {
            match op {
                PostfixOperation::Field { name, span } => {
                    let location = Self::location_of(span);
                    let key = fpas_sema::postfix_operation_lookup_key(op);
                    if let Some(info) = self.bound_methods.get(&key).cloned() {
                        self.emit_bound_method_from_receiver(&info, location)?;
                    } else if let Some(infos) = self.property_reads.get(&key).cloned() {
                        let info = infos.first().ok_or_else(|| {
                            internal_compiler_error(
                                "Property-read metadata has no getter entry.",
                                "Re-run compilation and report this internal compiler error.",
                                span.line,
                                span.column,
                            )
                        })?;
                        self.emit_property_get_from_receiver(info, location)?;
                    } else {
                        let idx = self.add_constant(Value::Str(name.clone()), location)?;
                        self.emit(Op::FieldGet(idx), location);
                    }
                }
                PostfixOperation::Index { index, span } => {
                    let location = Self::location_of(span);
                    self.compile_expr(index)?;
                    self.emit(Op::IndexGet, location);
                }
                PostfixOperation::MethodCall { args, span, .. } => {
                    self.compile_postfix_method_call(op, args, span)?;
                }
            }
        }
        Ok(())
    }

    fn compile_postfix_method_call(
        &mut self,
        op: &PostfixOperation,
        args: &[Expr],
        span: &fpas_lexer::Span,
    ) -> Result<(), CompileError> {
        let location = Self::location_of(span);
        let key = fpas_sema::postfix_operation_lookup_key(op);
        let Some(target) = self.method_calls.get(&key).cloned() else {
            return Err(internal_compiler_error(
                "Missing semantic metadata for postfix method call",
                "Internal compiler error: semantic analysis did not record a method target for this postfix call. Re-run compilation and report the source program.",
                span.line,
                span.column,
            ));
        };

        match target {
            fpas_sema::MethodCallTarget::Instance { qualified_name, .. } => {
                // Receiver is already on the stack from the preceding chain.
                if self.compile_std_library_call(&qualified_name, args, location)? {
                    return Ok(());
                }
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let total_args =
                    Self::checked_u8_at(args.len() + 1, "method call arguments", location)?;
                let name_idx = self.add_constant(Value::Str(qualified_name), location)?;
                self.emit(Op::Call(name_idx, total_args), location);
            }
            fpas_sema::MethodCallTarget::Static(_) => {
                return Err(internal_compiler_error(
                    "Postfix method call resolved as a static function",
                    "Internal compiler error: postfix chains only lower instance methods. Re-run compilation and report the source program.",
                    span.line,
                    span.column,
                ));
            }
        }
        Ok(())
    }
}
