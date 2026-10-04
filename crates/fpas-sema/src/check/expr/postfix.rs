//! Postfix expression chaining and invocation of callable values.
//!
//! **Documentation:** `docs/pascal/language/functions/postfix-chaining.md`

use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
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
    /// The final call must be a procedure; function results need an explicit consumer.
    /// **Documentation:** `docs/pascal/language/functions/postfix-chaining.md`
    pub(crate) fn check_postfix_statement(&mut self, expr: &Expr, span: Span) {
        let Expr::Postfix {
            base, operations, ..
        } = expr
        else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Expression statement must end with a call",
                "Call a procedure; consume function results or write `discard Function(...)`.",
                span,
            );
            self.check_expr(expr);
            return;
        };
        if !matches!(operations.last(), Some(PostfixOperation::Call { .. })) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Postfix expression statement must end with a call",
                "Call a procedure as a statement, or consume a value with `discard Expression;`.",
                span,
            );
            self.check_postfix_chain(base, operations, false);
            return;
        }
        let previous_error_count = self.errors.len();
        let result = self.check_postfix_chain(base, operations, true);
        if self.errors.len() == previous_error_count {
            self.require_consumed_call_result(&result, span);
        }
    }

    /// Check a postfix call that is spawned by `go`.
    pub(crate) fn check_postfix_go(
        &mut self,
        base: &Expr,
        operations: &[PostfixOperation],
        span: Span,
    ) -> Ty {
        if !matches!(operations.last(), Some(PostfixOperation::Call { .. })) {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "`go` requires a final call",
                "End the expression with `.Name(...)`.",
                span,
            );
            self.check_postfix_chain(base, operations, false);
            return Ty::Error;
        }
        self.check_postfix_chain(base, operations, true)
    }

    fn check_postfix_chain(
        &mut self,
        base: &Expr,
        operations: &[PostfixOperation],
        allow_final_procedure: bool,
    ) -> Ty {
        let mut ty = self.check_expr(base);
        let mut task_bound = self.expr_is_task_bound(Self::expr_lookup_key(base));
        for (index, operation) in operations.iter().enumerate() {
            if task_bound && matches!(operation, PostfixOperation::Call { .. }) {
                self.mark_expr_task_bound(Self::postfix_operation_lookup_key(operation));
            }
            let procedure_result_is_discarded =
                allow_final_procedure && index + 1 == operations.len();
            ty = self.check_postfix_operation(&ty, operation, procedure_result_is_discarded);
            self.projection_types.insert(
                Self::postfix_operation_lookup_key(operation),
                self.resolve_visible_type(&ty),
            );
            task_bound &= self.type_can_contain_callable(&ty);
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
                PostfixOperation::Call { args, .. } => self.check_args_only(args),
            }
            return Ty::Error;
        }

        let resolved = self.resolve_visible_type(ty);
        match operation {
            PostfixOperation::Call { args, span } => self.check_value_call(
                Self::postfix_operation_lookup_key(operation),
                "expression",
                &resolved,
                args,
                *span,
                procedure_result_is_discarded,
            ),
            PostfixOperation::Field { name, span } => {
                self.check_record_field_access(&resolved, name, *span)
            }
            PostfixOperation::Index { index, span } => {
                self.check_index_access(&resolved, index, *span)
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
