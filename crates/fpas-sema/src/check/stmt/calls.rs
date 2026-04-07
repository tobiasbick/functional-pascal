use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::{MethodKind, Ty};
use fpas_diagnostics::codes::{
    SEMA_AMBIGUOUS_IMPORTED_NAME, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    pub(super) fn check_call_stmt(&mut self, designator: &Designator, args: &[Expr], span: Span) {
        let name = Self::resolve_designator_name(designator);
        self.ensure_fq_std_unit_loaded(&name);

        if let Some(symbol) = self.scopes.lookup(&name)
            && symbol.kind == SymbolKind::BuiltinStd
        {
            let dispatch = self.builtin_std_dispatch_name(&name);
            let _ = crate::std_registry::check_builtin_std_call(self, &dispatch, args, span);
            return;
        }

        let symbol_ty = self.scopes.lookup(&name).map(|symbol| symbol.ty.clone());
        match symbol_ty {
            Some(Ty::Procedure(proc_ty)) => {
                self.check_procedure_call_args(&name, &proc_ty, args, span)
            }
            Some(Ty::Function(func_ty)) => {
                self.check_function_call_args(&name, &func_ty, args, span);
            }
            Some(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("`{name}` is not a procedure or function"),
                    "Only procedures and functions can be called.",
                    span,
                );
                self.check_args_only(args);
            }
            None => {
                if self.try_check_method_call_stmt(designator, args, span) {
                    return;
                }

                let (code, message, hint) = if let Some(ambiguous_hint) = self.ambiguous_hint(&name)
                {
                    (
                        SEMA_AMBIGUOUS_IMPORTED_NAME,
                        format!("Ambiguous name `{name}`"),
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
    }

    /// Try to resolve a call statement as a record or interface method call.
    fn try_check_method_call_stmt(
        &mut self,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> bool {
        if designator.parts.len() < 2 {
            return false;
        }

        let method_name = match designator.parts.last() {
            Some(DesignatorPart::Ident(name, _)) => name.clone(),
            _ => return false,
        };

        let receiver_designator = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };

        let receiver_ty = self.check_designator_expr(&receiver_designator);
        let resolved_receiver_ty = self.resolve_visible_type(&receiver_ty);

        // Record (concrete) receiver — static dispatch.
        let record_ty = match &resolved_receiver_ty {
            Ty::Record(record_ty) => record_ty.clone(),
            _ => return false,
        };

        let qualified = format!("{}.{}", record_ty.name, method_name);
        let method_kind = self.resolve_method_kind(&record_ty, &method_name, &qualified);
        let Some(method_kind) = method_kind else {
            return false;
        };

        self.method_calls
            .insert(crate::designator_lookup_key(designator), qualified.clone());

        self.check_stmt_method_kind(&qualified, &method_kind, args, span);
        true
    }

    fn check_stmt_method_kind(
        &mut self,
        qualified: &str,
        method_kind: &MethodKind,
        args: &[Expr],
        span: Span,
    ) {
        match method_kind {
            MethodKind::Procedure(proc_ty) => {
                let Some(visible_params) = proc_ty.params.get(1..) else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Method `{qualified}` must declare `Self` as its first parameter"),
                        "Declare the method as `procedure Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return;
                };
                self.check_method_call_args(
                    qualified,
                    &proc_ty.type_params,
                    visible_params,
                    args,
                    span,
                );
            }
            MethodKind::Function(func_ty) => {
                let Some(visible_params) = func_ty.params.get(1..) else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Method `{qualified}` must declare `Self` as its first parameter"),
                        "Declare the method as `function Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return;
                };
                self.check_method_call_args(
                    qualified,
                    &func_ty.type_params,
                    visible_params,
                    args,
                    span,
                );
            }
        }
    }
}
