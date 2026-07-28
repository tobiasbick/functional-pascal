//! Postfix expression chaining: `.Field`, `[Index]`, and `.Method(args)`.
//!
//! **Documentation:** `docs/pascal/language/functions/postfix-chaining.md`

use super::super::Checker;
use crate::check::MethodCallTarget;
use crate::types::{MethodKind, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;
use fpas_parser::{Expr, PostfixOperation};

impl Checker {
    /// Type-check a primary expression followed by one or more postfix suffixes.
    ///
    /// **Documentation:** `docs/pascal/language/functions/postfix-chaining.md`
    pub(crate) fn check_postfix_expr(
        &mut self,
        base: &Expr,
        operations: &[PostfixOperation],
    ) -> Ty {
        self.check_postfix_chain(base, operations, false)
    }

    /// Type-check a postfix chain used as a statement.
    ///
    /// The final method may be a procedure because its `Unit` result is discarded.
    /// **Documentation:** `docs/pascal/language/functions/postfix-chaining.md`
    pub(crate) fn check_postfix_statement(&mut self, expr: &Expr, span: Span) {
        let Expr::Postfix {
            base, operations, ..
        } = expr
        else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Expression statement must be a postfix method call",
                "End the statement with `.Method(...)`.",
                span,
            );
            self.check_expr(expr);
            return;
        };
        if !matches!(operations.last(), Some(PostfixOperation::MethodCall { .. })) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Postfix expression statement must end with a method call",
                "End the statement with `.Method(...)`; fields and indexes are values, not statements.",
                span,
            );
            self.check_postfix_chain(base, operations, false);
            return;
        }
        self.check_postfix_chain(base, operations, true);
    }

    fn check_postfix_chain(
        &mut self,
        base: &Expr,
        operations: &[PostfixOperation],
        allow_final_procedure: bool,
    ) -> Ty {
        let mut ty = self.check_expr(base);
        for (index, operation) in operations.iter().enumerate() {
            let procedure_result_is_discarded =
                allow_final_procedure && index + 1 == operations.len();
            ty = self.check_postfix_operation(&ty, operation, procedure_result_is_discarded);
        }
        ty
    }

    fn check_postfix_operation(
        &mut self,
        ty: &Ty,
        operation: &PostfixOperation,
        procedure_result_is_discarded: bool,
    ) -> Ty {
        if ty.is_error() {
            match operation {
                PostfixOperation::Field { .. } => {}
                PostfixOperation::Index { index, .. } => {
                    self.check_expr(index);
                }
                PostfixOperation::MethodCall { args, .. } => self.check_args_only(args),
            }
            return Ty::Error;
        }

        let resolved = self.resolve_visible_type(ty);
        match operation {
            PostfixOperation::Field { name, span } => {
                let key = Self::postfix_operation_lookup_key(operation);
                self.check_record_member_access(
                    &resolved,
                    name,
                    *span,
                    Some((key, 0)),
                    Some((key, 0)),
                )
            }
            PostfixOperation::Index { index, span } => {
                self.check_index_access(&resolved, index, *span)
            }
            PostfixOperation::MethodCall { name, args, span } => self.check_postfix_method_call(
                &resolved,
                name,
                args,
                *span,
                operation,
                procedure_result_is_discarded,
            ),
        }
    }

    fn check_postfix_method_call(
        &mut self,
        receiver_ty: &Ty,
        method_name: &str,
        args: &[Expr],
        span: Span,
        operation: &PostfixOperation,
        procedure_result_is_discarded: bool,
    ) -> Ty {
        let Ty::Record(record_ty) = receiver_ty else {
            if !receiver_ty.is_error() {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("`.{method_name}(...)` requires a record value"),
                    "Only records support instance method calls after an expression.",
                    span,
                );
            }
            self.check_args_only(args);
            return Ty::Error;
        };

        if self.reject_private_record_member(record_ty, method_name, span) {
            self.check_args_only(args);
            return Ty::Error;
        }

        let qualified = format!("{}.{}", record_ty.name, method_name);
        let op_key = Self::postfix_operation_lookup_key(operation);

        if let Some(routine_kind) = self.static_routine_kind_on_record(record_ty, method_name) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{method_name}` is a static {routine_kind} and must be called through the type `{}.{}`",
                    record_ty.name, method_name
                ),
                format!(
                    "Write `{}.{}(...)` instead of calling it on a value.",
                    record_ty.name, method_name
                ),
                span,
            );
            self.check_args_only(args);
            return Ty::Error;
        }

        let Some(method_kind) = self.resolve_method_kind(record_ty, method_name, &qualified) else {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!("Record `{}` has no method `{method_name}`", record_ty.name),
                "Check the record declaration or use a field without parentheses.",
                span,
            );
            self.check_args_only(args);
            return Ty::Error;
        };

        self.method_calls.insert(
            op_key,
            MethodCallTarget::Instance {
                qualified_name: qualified.clone(),
                receiver_reads: Vec::new(),
            },
        );

        match method_kind {
            MethodKind::Function(func_ty) => {
                let Some(visible_params) = func_ty.params.get(1..) else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Record method `{qualified}` must declare `Self` as its first parameter"
                        ),
                        "Declare the method as `function Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return Ty::Error;
                };
                let inferred = self.check_method_call_args(
                    &qualified,
                    &func_ty.type_params,
                    visible_params,
                    args,
                    span,
                );
                Self::substitute_type_params(&func_ty.return_type, &inferred)
            }
            MethodKind::Procedure(proc_ty) => {
                let Some(visible_params) = proc_ty.params.get(1..) else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Record method `{qualified}` must declare `Self` as its first parameter"
                        ),
                        "Declare the method as `procedure Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return Ty::Error;
                };
                self.check_method_call_args(
                    &qualified,
                    &proc_ty.type_params,
                    visible_params,
                    args,
                    span,
                );
                if procedure_result_is_discarded {
                    Ty::Unit
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Method procedure `{qualified}` does not return a value"),
                        "Use a method function instead if you need a return value, or make the procedure the final call of a statement.",
                        span,
                    );
                    Ty::Error
                }
            }
        }
    }

    /// Stable identity key for a postfix operation node in the AST.
    ///
    /// Uses the memory address of the `PostfixOperation` reference. Sound because the AST is
    /// immutable for the whole compile pipeline; keys must match between sema and codegen.
    ///
    /// **Documentation:** `docs/pascal/language/functions/README.md`
    #[must_use]
    pub fn postfix_operation_lookup_key(operation: &PostfixOperation) -> usize {
        std::ptr::from_ref(operation) as usize
    }
}
