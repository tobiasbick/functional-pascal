use super::super::Checker;
use crate::check::expr::MethodCallSite;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_AMBIGUOUS_IMPORTED_NAME, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

impl Checker {
    pub(super) fn check_call_stmt(&mut self, designator: &Designator, args: &[Expr], span: Span) {
        let name = Self::resolve_designator_name(designator);
        self.ensure_fq_std_unit_loaded(&name);

        if let Some(symbol) = self.scopes.lookup(&name) {
            let kind = symbol.kind;
            let ty = symbol.ty.clone();
            if self.reject_instance_method_through_type(designator, span) {
                self.check_args_only(args);
                return;
            }

            let dispatch = self.builtin_std_dispatch_name(&name);
            if dispatch.starts_with("Std.") {
                self.intrinsic_calls
                    .insert(crate::designator_lookup_key(designator), dispatch.clone());
            }
            if kind == SymbolKind::BuiltinStd {
                let _ = crate::std_registry::check_builtin_std_call(self, &dispatch, args, span);
                return;
            }

            match ty {
                Ty::Procedure(proc_ty) => {
                    self.check_procedure_call_args(&name, &proc_ty, args, span);
                    return;
                }
                Ty::Function(func_ty) => {
                    self.check_function_call_args(&name, &func_ty, args, span);
                    return;
                }
                _ => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("`{name}` is not a procedure or function"),
                        "Only procedures and functions can be called.",
                        span,
                    );
                    self.check_args_only(args);
                    return;
                }
            }
        }

        if !self.designator_has_unit_prefix(designator) {
            let previous_error_count = self.errors.len();
            if self
                .try_check_method_call_like(MethodCallSite::Statement, designator, args, span)
                .is_some()
            {
                return;
            }
            if self.errors.len() != previous_error_count {
                self.check_args_only(args);
                return;
            }

            if self
                .try_check_fluent_designator(
                    crate::designator_lookup_key(designator),
                    designator,
                    args,
                    span,
                    true,
                )
                .is_some()
            {
                return;
            }
        }

        let (code, message, hint) = if let Some(ambiguous_hint) = self.ambiguous_hint(&name) {
            (
                SEMA_AMBIGUOUS_IMPORTED_NAME,
                format!("Ambiguous imported symbol `{name}`"),
                ambiguous_hint,
            )
        } else {
            (
                SEMA_UNKNOWN_NAME,
                format!("Unknown procedure `{name}`"),
                self.hint_unknown_callable(&name),
            )
        };

        self.error_with_code(code, message, hint, span);
        self.check_args_only(args);
    }
}
