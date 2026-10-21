//! Type-checking for anonymous function / procedure expressions (closures).
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use super::super::Checker;
use super::super::closures::{CaptureOwner, closure_info_from_captures};
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
        span: Span,
    ) -> Ty {
        self.check_unique_formal_param_names(params);
        let params_ty = self.resolve_formal_params(params);
        let return_ty = return_type.map(|te| self.resolve_type_expr(te));

        let key = Self::expr_lookup_key(expr);
        // This private lexical type scope must not expose the AST identity address.
        // Executable names are numbered separately in lowering's source-order walk.
        let scope_name = format!("$closure_at_{}", span.offset);

        let analysis = self.check_routine_body_collecting_captures(
            &scope_name,
            &[],
            &params_ty,
            params,
            return_ty.clone(),
            super::super::contracts::RoutineBody {
                body,
                requires: &[],
                ensures: &[],
            },
        );
        self.reject_closure_var_captures(&analysis.captures, span);
        self.defer_closure_var_captures(&analysis.pending_routines, span);
        self.register_capture_dependencies(
            CaptureOwner::Closure(key),
            analysis.scope_index,
            &analysis.pending_routines,
        );
        let info = closure_info_from_captures(analysis.captures);
        self.set_expr_capability(key, info.capability);
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
