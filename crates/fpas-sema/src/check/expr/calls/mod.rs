mod methods;

use super::super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{
    SEMA_AMBIGUOUS_IMPORTED_NAME, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};
use fpas_lexer::Span;
use fpas_parser::{Designator, Expr};

/// Result of resolving a call target before checking argument types.
pub(in crate::check::expr) enum CallResolution {
    /// Resolved to a known symbol (kind + type).
    Symbol { kind: SymbolKind, ty: Ty },
    /// Resolved as a method call — the return type is already fully checked.
    MethodResult(Ty),
    /// Resolution failed (error already reported, args already checked).
    Failed,
}

impl Checker {
    /// Resolve a call target: symbol lookup → method fallback → ambiguous/unknown error.
    pub(in crate::check::expr) fn resolve_call_target(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> CallResolution {
        let name = Self::resolve_designator_name(designator);
        self.ensure_fq_std_unit_loaded(&name);

        if let Some(symbol) = self.scopes.lookup(&name) {
            return CallResolution::Symbol {
                kind: symbol.kind,
                ty: symbol.ty.clone(),
            };
        }

        let method_result = if allow_procedure_result {
            self.try_check_method_go_call(call_expr, designator, args, span)
        } else {
            self.try_check_method_call(call_expr, designator, args, span)
        };
        if let Some(result) = method_result {
            return CallResolution::MethodResult(result);
        }

        if let Some(hint) = self.ambiguous_hint(&name) {
            self.error_with_code(
                SEMA_AMBIGUOUS_IMPORTED_NAME,
                format!("Ambiguous name `{name}`"),
                hint,
                span,
            );
            self.check_args_only(args);
            return CallResolution::Failed;
        }

        let hint = self.hint_unknown_callable(&name);
        self.error_with_code(
            SEMA_UNKNOWN_NAME,
            format!("Unknown function or procedure `{name}`"),
            hint,
            span,
        );
        self.check_args_only(args);
        CallResolution::Failed
    }

    pub(super) fn check_call_expr(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        match self.resolve_call_target(call_expr, designator, args, span, false) {
            CallResolution::Symbol { kind, ty } => {
                let name = Self::resolve_designator_name(designator);
                self.check_known_call_symbol(&name, kind, ty, args, span)
            }
            CallResolution::MethodResult(ty) => ty,
            CallResolution::Failed => Ty::Error,
        }
    }

    fn check_known_call_symbol(
        &mut self,
        name: &str,
        symbol_kind: SymbolKind,
        symbol_ty: Ty,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        if symbol_kind == SymbolKind::BuiltinStd {
            let dispatch = self.builtin_std_dispatch_name(name);
            return crate::std_registry::check_builtin_std_call(self, &dispatch, args, span);
        }

        if symbol_kind == SymbolKind::EnumVariantConstructor {
            return self.check_enum_variant_constructor_call(name, &symbol_ty, args, span);
        }

        match &symbol_ty {
            Ty::Function(func_ty) => {
                self.check_function_call_args(name, func_ty, args, span);
                *func_ty.return_type.clone()
            }
            Ty::Procedure(_) => {
                self.check_args_only(args);
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Procedure `{name}` does not return a value"),
                    "Use a function instead if you need a return value.",
                    span,
                );
                Ty::Error
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("`{name}` is not callable"),
                    "Only functions and procedures can be called.",
                    span,
                );
                self.check_args_only(args);
                Ty::Error
            }
        }
    }

    pub(super) fn check_known_go_call_symbol(
        &mut self,
        name: &str,
        symbol_kind: SymbolKind,
        symbol_ty: Ty,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        if symbol_kind == SymbolKind::BuiltinStd {
            let dispatch = self.builtin_std_dispatch_name(name);
            return crate::std_registry::check_builtin_std_call(self, &dispatch, args, span);
        }

        if symbol_kind == SymbolKind::EnumVariantConstructor {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`go` requires a function or procedure call, but `{name}` constructs a value"
                ),
                "Spawn a named function, procedure, method call, or callable variable.",
                span,
            );
            self.check_args_only(args);
            return Ty::Error;
        }

        match &symbol_ty {
            Ty::Function(func_ty) => {
                self.check_function_call_args(name, func_ty, args, span);
                *func_ty.return_type.clone()
            }
            Ty::Procedure(proc_ty) => {
                self.check_procedure_call_args(name, proc_ty, args, span);
                Ty::Unit
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "`go` requires a function or procedure call, but `{name}` is not callable"
                    ),
                    "Use `go FunctionName(args)` or `go SomeCallable(args)`.",
                    span,
                );
                self.check_args_only(args);
                Ty::Error
            }
        }
    }

    fn check_enum_variant_constructor_call(
        &mut self,
        name: &str,
        enum_ty: &Ty,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        if let Ty::Enum(enum_def) = enum_ty {
            let variant_name = name.rsplit('.').next().unwrap_or(name);
            if let Some(variant) = enum_def
                .variants
                .iter()
                .find(|v| v.name.eq_ignore_ascii_case(variant_name))
            {
                if args.len() != variant.fields.len() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "`{name}` expects {} argument(s), got {}",
                            variant.fields.len(),
                            args.len()
                        ),
                        format!(
                            "Provide values for: {}",
                            variant
                                .fields
                                .iter()
                                .map(|(field_name, _)| field_name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        span,
                    );
                }

                for (arg, (_field_name, field_ty)) in args.iter().zip(variant.fields.iter()) {
                    let arg_ty = self.check_expr(arg);
                    self.check_type_compat(
                        field_ty,
                        &arg_ty,
                        &format!("`{name}` argument"),
                        super::super::spans::expr_span(arg),
                    );
                }

                for arg in args.iter().skip(variant.fields.len()) {
                    self.check_expr(arg);
                }

                return enum_ty.clone();
            }
        }

        self.check_args_only(args);
        enum_ty.clone()
    }
}
