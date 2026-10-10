mod fluent;
mod indexed;
mod methods;
mod native;

pub(in crate::check) use fluent::FluentCall;
pub(in crate::check) use methods::MethodCallSite;

use super::super::Checker;
use crate::check::calls::CallTarget;
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
        if let Some(result) = self.try_check_indexed_callable(
            Self::expr_lookup_key(call_expr),
            designator,
            args,
            span,
            allow_procedure_result,
        ) {
            return CallResolution::MethodResult(result);
        }
        if let Some(result) =
            self.try_check_native_factory(Self::expr_lookup_key(call_expr), designator, args, span)
        {
            return CallResolution::MethodResult(result);
        }
        let name = Self::resolve_designator_name(designator);
        if name
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Std."))
            && let Some(hint) = crate::std_registry::native_migration_hint(&name)
        {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!("Removed free type operation `{name}`"),
                hint,
                span,
            );
            self.check_args_only(args);
            return CallResolution::Failed;
        }
        self.ensure_fq_std_unit_loaded(&name);

        if let Some(symbol) = self.scopes.lookup(&name) {
            let kind = symbol.kind;
            let ty = symbol.ty.clone();
            if self.reject_instance_method_through_type(designator, span) {
                self.check_args_only(args);
                return CallResolution::Failed;
            }
            return CallResolution::Symbol { kind, ty };
        }

        if !self.designator_has_unit_prefix(designator) {
            let previous_error_count = self.errors.len();
            let method_result = if allow_procedure_result {
                self.try_check_method_go_call(call_expr, designator, args, span)
            } else {
                self.try_check_method_call(call_expr, designator, args, span)
            };
            if let Some(result) = method_result {
                return CallResolution::MethodResult(result);
            }
            if self.errors.len() != previous_error_count {
                self.check_args_only(args);
                return CallResolution::Failed;
            }

            if let Some(result) = self.try_check_fluent_designator(
                Self::expr_lookup_key(call_expr),
                designator,
                args,
                span,
                allow_procedure_result,
            ) {
                return CallResolution::MethodResult(result);
            }
        }

        if let Some(hint) = self.ambiguous_call_hint(&name, args.len()) {
            self.error_with_code(
                SEMA_AMBIGUOUS_IMPORTED_NAME,
                format!("Ambiguous imported symbol `{name}`"),
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

    /// Check an ordinary call, passing expected types to record constructors.
    pub(super) fn check_call_expr(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        match self.resolve_call_target(call_expr, designator, args, span, false) {
            CallResolution::Symbol { kind, ty } => {
                let name = Self::resolve_designator_name(designator);
                self.check_known_call_symbol(
                    Self::expr_lookup_key(call_expr),
                    &name,
                    kind,
                    ty,
                    args,
                    span,
                    expected,
                )
            }
            CallResolution::MethodResult(ty) => ty,
            CallResolution::Failed => Ty::Error,
        }
    }

    fn check_known_call_symbol(
        &mut self,
        call_key: usize,
        name: &str,
        symbol_kind: SymbolKind,
        symbol_ty: Ty,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        if symbol_kind == SymbolKind::EnumVariantConstructor {
            return self.check_enum_construction(name, &symbol_ty, args, span, expected);
        }
        if symbol_kind == SymbolKind::Type
            && let Ty::Record(record) = self.resolve_visible_type(&symbol_ty)
        {
            return self.check_record_construction(call_key, &record, args, span, expected);
        }
        if symbol_kind == SymbolKind::Type
            && let Some(ty) = self.try_check_type_conversion(call_key, name, &symbol_ty, args, span)
        {
            return ty;
        }
        let dispatch = self.builtin_std_dispatch_name(name);
        if dispatch.starts_with("Std.") {
            self.intrinsic_calls.insert(call_key, dispatch.clone());
        }
        if symbol_kind == SymbolKind::BuiltinStd {
            return self.check_builtin_std_call_positional(name, &dispatch, args, span);
        }

        match &symbol_ty {
            Ty::Function(func_ty) => {
                let inferred = self.check_function_call_args(
                    name,
                    func_ty,
                    CallTarget::for_symbol(symbol_kind),
                    args,
                    span,
                );
                Self::substitute_type_params(&func_ty.return_type, &inferred)
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
            return self.check_builtin_std_call_positional(name, &dispatch, args, span);
        }

        if symbol_kind == SymbolKind::EnumVariantConstructor
            || (symbol_kind == SymbolKind::Type
                && matches!(
                    self.resolve_visible_type(&symbol_ty),
                    Ty::Record(_) | Ty::Distinct(_)
                ))
        {
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
                let inferred = self.check_function_call_args(
                    name,
                    func_ty,
                    CallTarget::for_symbol(symbol_kind),
                    args,
                    span,
                );
                Self::substitute_type_params(&func_ty.return_type, &inferred)
            }
            Ty::Procedure(proc_ty) => {
                self.check_procedure_call_args(
                    name,
                    proc_ty,
                    CallTarget::for_symbol(symbol_kind),
                    args,
                    span,
                );
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

    /// Checks a polymorphic standard-library call, which takes positional arguments only.
    pub(in crate::check) fn check_builtin_std_call_positional(
        &mut self,
        name: &str,
        dispatch: &str,
        args: &[Expr],
        span: Span,
    ) -> Ty {
        if self.reject_named_arguments(
            args,
            name,
            "This standard-library operation has no declared parameter names; pass its arguments by position.",
        ) {
            self.check_args_only(args);
            return Ty::Error;
        }
        crate::std_registry::check_builtin_std_call(self, dispatch, args, span)
    }
}
