use super::super::super::Checker;
use crate::check::MethodCallTarget;
use crate::scope::SymbolKind;
use crate::types::{GenericParamDef, MethodKind, ParamTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};
use std::collections::HashMap;

impl Checker {
    /// Try to resolve `designator(args)` as a record instance or static member call.
    /// Returns `Some(return_ty)` when the call is a record member invocation.
    pub(super) fn try_check_method_call(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Option<Ty> {
        self.try_check_method_call_like(call_expr, designator, args, span, false)
    }

    pub(in crate::check::expr) fn try_check_method_go_call(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
    ) -> Option<Ty> {
        self.try_check_method_call_like(call_expr, designator, args, span, true)
    }

    pub(in crate::check) fn resolve_method_kind(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
        qualified: &str,
    ) -> Option<MethodKind> {
        if let Some((_, method_kind)) = record_ty
            .methods
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(method_kind.clone());
        }

        let symbol = self.scopes.lookup(qualified)?;
        match &symbol.ty {
            Ty::Function(function_ty) => Some(MethodKind::Function(function_ty.clone())),
            Ty::Procedure(procedure_ty) => Some(MethodKind::Procedure(procedure_ty.clone())),
            _ => None,
        }
    }

    pub(in crate::check) fn resolve_static_function(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
        _qualified: &str,
    ) -> Option<crate::types::FunctionTy> {
        if let Some((_, function_ty)) = record_ty
            .static_functions
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(function_ty.clone());
        }

        // RecordTy clones on values may omit the table; consult the type symbol.
        if let Some(symbol) = self.scopes.lookup(&record_ty.name)
            && let Ty::Record(stored) = &symbol.ty
            && let Some((_, function_ty)) = stored
                .static_functions
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return Some(function_ty.clone());
        }
        None
    }

    fn is_static_function_symbol(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
    ) -> bool {
        if record_ty
            .static_functions
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(method_name))
        {
            return true;
        }
        if let Some(symbol) = self.scopes.lookup(&record_ty.name)
            && let Ty::Record(stored) = &symbol.ty
        {
            return stored
                .static_functions
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(method_name));
        }
        false
    }

    pub(in crate::check) fn check_method_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        visible_params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_method_call_args_with_hint(
            name,
            type_params,
            visible_params,
            args,
            span,
            "Check the number of arguments (Self is implicit).",
        )
    }

    pub(in crate::check) fn check_static_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) -> HashMap<String, Ty> {
        self.check_method_call_args_with_hint(
            name,
            type_params,
            params,
            args,
            span,
            "Check the number of arguments.",
        )
    }

    fn check_method_call_args_with_hint(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        visible_params: &[ParamTy],
        args: &[Expr],
        span: Span,
        arity_hint: &str,
    ) -> HashMap<String, Ty> {
        if visible_params.len() != args.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Method `{name}` expects {} arguments, got {}",
                    visible_params.len(),
                    args.len()
                ),
                arity_hint,
                span,
            );
        }

        let mut arg_types = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            let arg_ty = self.check_expr(arg);
            if let Some(param) = visible_params.get(index) {
                self.check_type_compat(
                    &param.ty,
                    &arg_ty,
                    &format!("argument {}", index + 1),
                    span,
                );
            }
            arg_types.push(arg_ty);
        }

        self.validate_routine_constraints(type_params, visible_params, &arg_types, span)
    }

    /// True when the designator names a type symbol (possibly qualified).
    pub(in crate::check) fn designator_denotes_type(&self, designator: &Designator) -> bool {
        let name = Self::resolve_designator_name(designator);
        self.scopes
            .lookup(&name)
            .is_some_and(|symbol| matches!(symbol.kind, SymbolKind::Type))
    }

    /// Reject `Type.InstanceMethod(...)` looked up as a free function through the type name.
    pub(in crate::check) fn reject_instance_method_through_type(
        &mut self,
        designator: &Designator,
        span: Span,
    ) -> bool {
        if designator.parts.len() < 2 {
            return false;
        }
        let method_name = match designator.parts.last() {
            Some(DesignatorPart::Ident(name, _)) => name.clone(),
            _ => return false,
        };
        let receiver = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };
        if !self.designator_denotes_type(&receiver) {
            return false;
        }
        let receiver_ty = self.check_designator_expr(&receiver);
        let Ty::Record(record_ty) = self.resolve_visible_type(&receiver_ty) else {
            return false;
        };
        let is_instance = record_ty
            .methods
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(&method_name));
        if !is_instance {
            return false;
        }
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "`{method_name}` is an instance method and must be called through a `{}.…` value",
                record_ty.name
            ),
            format!(
                "Call it as `Value.{method_name}(...)` where `Value` has type `{}`.",
                record_ty.name
            ),
            span,
        );
        true
    }

    fn try_check_method_call_like(
        &mut self,
        call_expr: &Expr,
        designator: &Designator,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Option<Ty> {
        if designator.parts.len() < 2 {
            return None;
        }

        let method_name = match designator.parts.last()? {
            DesignatorPart::Ident(name, _) => name.clone(),
            _ => return None,
        };

        let receiver_designator = Designator {
            parts: designator.parts[..designator.parts.len() - 1].to_vec(),
            span: designator.span,
        };

        let through_type = self.designator_denotes_type(&receiver_designator);
        // Resolve the full receiver path (supports qualified aliases like `Geom.Api.Point`).
        let receiver_ty = self.check_designator_expr(&receiver_designator);
        let resolved_receiver_ty = self.resolve_visible_type(&receiver_ty);

        let record_ty = match &resolved_receiver_ty {
            Ty::Record(record_ty) => record_ty.clone(),
            _ => return None,
        };

        let qualified = format!("{}.{}", record_ty.name, method_name);
        let call_key = Self::expr_lookup_key(call_expr);

        if through_type {
            if let Some(func_ty) =
                self.resolve_static_function(&record_ty, &method_name, &qualified)
            {
                self.method_calls
                    .insert(call_key, MethodCallTarget::Static(qualified.clone()));
                let inferred = self.check_static_call_args(
                    &qualified,
                    &func_ty.type_params,
                    &func_ty.params,
                    args,
                    span,
                );
                return Some(Self::substitute_type_params(
                    &func_ty.return_type,
                    &inferred,
                ));
            }

            if self
                .resolve_method_kind(&record_ty, &method_name, &qualified)
                .is_some()
            {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "`{method_name}` is an instance method and must be called through a `{}.…` value",
                        record_ty.name
                    ),
                    format!(
                        "Call it as `Value.{method_name}(...)` where `Value` has type `{}`.",
                        record_ty.name
                    ),
                    span,
                );
                self.check_args_only(args);
                return Some(Ty::Error);
            }
            return None;
        }

        // Value receiver: instance methods only.
        if self.is_static_function_symbol(&record_ty, &method_name)
            || record_ty
                .static_functions
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(&method_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{method_name}` is a static function and must be called through the type `{}.{}`",
                    record_ty.name, method_name
                ),
                format!(
                    "Write `{}.{}(...)` instead of calling it on a value.",
                    record_ty.name, method_name
                ),
                span,
            );
            self.check_args_only(args);
            return Some(Ty::Error);
        }

        let method_kind = self.resolve_method_kind(&record_ty, &method_name, &qualified)?;

        self.method_calls
            .insert(call_key, MethodCallTarget::Instance(qualified.clone()));

        match &method_kind {
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
                    return Some(Ty::Error);
                };
                let inferred = self.check_method_call_args(
                    &qualified,
                    &func_ty.type_params,
                    visible_params,
                    args,
                    span,
                );
                Some(Self::substitute_type_params(
                    &func_ty.return_type,
                    &inferred,
                ))
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
                    return Some(Ty::Error);
                };
                self.check_method_call_args(
                    &qualified,
                    &proc_ty.type_params,
                    visible_params,
                    args,
                    span,
                );
                if allow_procedure_result {
                    Some(Ty::Unit)
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Method procedure `{qualified}` does not return a value"),
                        "Use a method function instead if you need a return value.",
                        span,
                    );
                    Some(Ty::Error)
                }
            }
        }
    }
}
