//! Type-checking for anonymous function / procedure expressions (closures).
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use super::super::Checker;
use super::super::closures::closure_info_from_captures;
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_lexer::Span;
use fpas_parser::{Expr, FormalParam, FuncBody, TypeExpr};

impl Checker {
    /// Type-check a closure expression like a nested routine and record its captures.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) fn check_closure_expr(
        &mut self,
        expr: &Expr,
        is_function: bool,
        params: &[FormalParam],
        return_type: Option<&TypeExpr>,
        body: &FuncBody,
        _span: Span,
    ) -> Ty {
        self.check_unique_formal_param_names(params);
        let params_ty = self.resolve_formal_params(params);
        let return_ty = return_type.map(|te| self.resolve_type_expr(te));

        let key = Self::expr_lookup_key(expr);
        let synthetic_name = format!("$closure_{key}");

        let captures = self.check_routine_body_collecting_captures(
            &synthetic_name,
            &[],
            &params_ty,
            return_ty.clone(),
            body,
        );
        let info = closure_info_from_captures(synthetic_name, captures);
        if info.task_bound {
            self.mark_expr_task_bound(key);
        }
        self.closure_infos.insert(key, info);

        if is_function {
            Ty::Function(FunctionTy {
                type_params: Vec::new(),
                params: params_ty,
                return_type: Box::new(return_ty.unwrap_or(Ty::Error)),
                variadic: false,
            })
        } else {
            Ty::Procedure(ProcedureTy {
                type_params: Vec::new(),
                params: params_ty,
                variadic: false,
            })
        }
    }
}
