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
    /// Checks calls in statement position, including procedure operands rejected by discard.
    pub(crate) fn check_call_stmt(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        let name = Self::resolve_designator_name(designator);
        self.ensure_fq_std_unit_loaded(&name);

        if let Some(symbol) = self.scopes.lookup(&name) {
            let kind = symbol.kind;
            let ty = symbol.ty.clone();
            if self.reject_instance_method_through_type(designator, span) {
                self.check_args_only(args);
                return Ty::Error;
            }

            let dispatch = self.builtin_std_dispatch_name(&name);
            if dispatch.starts_with("Std.") {
                self.intrinsic_calls
                    .insert(crate::designator_lookup_key(designator), dispatch.clone());
            }
            if kind == SymbolKind::BuiltinStd {
                return crate::std_registry::check_builtin_std_call(self, &dispatch, args, span);
            }

            match ty {
                Ty::Procedure(proc_ty) => {
                    self.check_procedure_call_args(&name, &proc_ty, args, span);
                    return Ty::Unit;
                }
                Ty::Function(func_ty) => {
                    let inferred = self.check_function_call_args(&name, &func_ty, args, span);
                    return Self::substitute_type_params(&func_ty.return_type, &inferred);
                }
                _ => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("`{name}` is not a procedure or function"),
                        "Only procedures and functions can be called.",
                        span,
                    );
                    self.check_args_only(args);
                    return Ty::Error;
                }
            }
        }

        if !self.designator_has_unit_prefix(designator) {
            let previous_error_count = self.errors.len();
            if let Some(ty) =
                self.try_check_method_call_like(MethodCallSite::Statement, designator, args, span)
            {
                return ty;
            }
            if self.errors.len() != previous_error_count {
                self.check_args_only(args);
                return Ty::Error;
            }

            if let Some(ty) = self.try_check_fluent_designator(
                crate::designator_lookup_key(designator),
                designator,
                args,
                span,
                true,
            ) {
                return ty;
            }
        }

        let (code, message, hint) =
            if let Some(ambiguous_hint) = self.ambiguous_call_hint(&name, args.len()) {
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
        Ty::Error
    }
}
