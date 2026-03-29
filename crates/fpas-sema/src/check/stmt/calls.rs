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
                // Try method call dispatch.
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

    /// Try to resolve a call statement as a record method call.
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
        let Ty::Record(record_ty) = &receiver_ty else {
            return false;
        };

        let method = record_ty
            .methods
            .iter()
            .find(|(name, _)| name == &method_name);

        // Fallback: look up TypeName.MethodName in scope (handles Self inside method bodies
        // where the record type doesn't yet have methods populated).
        let qualified = format!("{}.{}", record_ty.name, method_name);
        let method_kind = if let Some((_, mk)) = method {
            mk.clone()
        } else if let Some(sym) = self.scopes.lookup(&qualified) {
            match &sym.ty {
                Ty::Function(f) => MethodKind::Function(f.clone()),
                Ty::Procedure(p) => MethodKind::Procedure(p.clone()),
                _ => return false,
            }
        } else {
            return false;
        };

        // Record for the compiler that this call is a method invocation.
        let key = std::ptr::from_ref(designator) as usize;
        self.method_calls.insert(key, qualified.clone());

        match &method_kind {
            MethodKind::Procedure(proc_ty) => {
                // Check visible params (excluding Self).
                let Some(visible_params) = proc_ty.params.get(1..) else {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!(
                            "Record method `{qualified}` must declare `Self` as its first parameter"
                        ),
                        "Declare the method as `procedure Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return true;
                };
                if visible_params.len() != args.len() {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT,
                        format!(
                            "Method `{qualified}` expects {} arguments, got {}",
                            visible_params.len(),
                            args.len()
                        ),
                        "Check the number of arguments (Self is implicit).",
                        span,
                    );
                }
                for (i, arg) in args.iter().enumerate() {
                    let arg_ty = self.check_expr(arg);
                    if let Some(param) = visible_params.get(i) {
                        self.check_type_compat(
                            &param.ty,
                            &arg_ty,
                            &format!("argument {}", i + 1),
                            span,
                        );
                    }
                }
            }
            MethodKind::Function(func_ty) => {
                let Some(visible_params) = func_ty.params.get(1..) else {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!(
                            "Record method `{qualified}` must declare `Self` as its first parameter"
                        ),
                        "Declare the method as `function Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return true;
                };
                if visible_params.len() != args.len() {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_WRONG_ARGUMENT_COUNT,
                        format!(
                            "Method `{qualified}` expects {} arguments, got {}",
                            visible_params.len(),
                            args.len()
                        ),
                        "Check the number of arguments (Self is implicit).",
                        span,
                    );
                }
                for (i, arg) in args.iter().enumerate() {
                    let arg_ty = self.check_expr(arg);
                    if let Some(param) = visible_params.get(i) {
                        self.check_type_compat(
                            &param.ty,
                            &arg_ty,
                            &format!("argument {}", i + 1),
                            span,
                        );
                    }
                }
            }
        }

        true
    }
}
