use super::super::super::Checker;
use crate::types::{GenericParamDef, MethodKind, ParamTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_WRONG_ARGUMENT_COUNT};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Try to resolve `designator(args)` as a record method call.
    /// Returns `Some(return_ty)` if the last designator part is a method on the receiver record.
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

    fn resolve_method_kind(
        &self,
        record_ty: &crate::types::RecordTy,
        method_name: &str,
        qualified: &str,
    ) -> Option<MethodKind> {
        if let Some((_, method_kind)) = record_ty
            .methods
            .iter()
            .find(|(name, _)| name == method_name)
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

    fn check_method_call_args(
        &mut self,
        name: &str,
        type_params: &[GenericParamDef],
        visible_params: &[ParamTy],
        args: &[Expr],
        span: Span,
    ) {
        if visible_params.len() != args.len() {
            self.error_with_code(
                SEMA_WRONG_ARGUMENT_COUNT,
                format!(
                    "Method `{name}` expects {} arguments, got {}",
                    visible_params.len(),
                    args.len()
                ),
                "Check the number of arguments (Self is implicit).",
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

        self.validate_routine_constraints(type_params, visible_params, &arg_types, span);
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

        let Some(DesignatorPart::Ident(base_name, _)) = designator.parts.first() else {
            return None;
        };
        if self.scopes.lookup(base_name).is_none() {
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

        let receiver_ty = self.check_designator_expr(&receiver_designator);
        let resolved_receiver_ty = self.resolve_visible_type(&receiver_ty);

        // ── Interface receiver — virtual dispatch ─────────────────────────────────
        if let Ty::Interface(iface) = &resolved_receiver_ty {
            return self.try_check_interface_method_call(
                call_expr,
                iface.clone(),
                &method_name,
                args,
                span,
                allow_procedure_result,
            );
        }

        // ── Record (concrete) receiver — static dispatch ──────────────────────────
        let record_ty = match &resolved_receiver_ty {
            Ty::Record(record_ty) => record_ty.clone(),
            Ty::Ref(inner) => {
                let inner_ty = self.resolve_visible_type(inner);
                let Ty::Record(record_ty) = inner_ty else {
                    return None;
                };
                record_ty
            }
            _ => return None,
        };

        let qualified = format!("{}.{}", record_ty.name, method_name);
        let method_kind = self.resolve_method_kind(&record_ty, &method_name, &qualified)?;

        let call_key = Self::expr_lookup_key(call_expr);
        self.method_calls.insert(call_key, qualified.clone());

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
                self.check_method_call_args(&qualified, &func_ty.type_params, visible_params, args, span);
                Some(*func_ty.return_type.clone())
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
                self.check_method_call_args(&qualified, &proc_ty.type_params, visible_params, args, span);
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

    /// Handle a method call on an interface-typed receiver.
    ///
    /// Records the call in `interface_dispatch` so the compiler can emit `CallVirtual`.
    fn try_check_interface_method_call(
        &mut self,
        call_expr: &Expr,
        iface: crate::types::InterfaceTy,
        method_name: &str,
        args: &[Expr],
        span: Span,
        allow_procedure_result: bool,
    ) -> Option<Ty> {
        let method_kind = iface
            .methods
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(method_name))?
            .1
            .clone();

        let call_key = Self::expr_lookup_key(call_expr);
        // Store the interface-qualified name in method_calls so the receiver is compiled.
        let qualified = format!("{}.{}", iface.name, method_name);
        self.method_calls.insert(call_key, qualified);
        // Mark this call for virtual dispatch; the compiler uses the unqualified method name.
        self.interface_dispatch
            .insert(call_key, method_name.to_owned());

        match &method_kind {
            MethodKind::Function(func_ty) => {
                let visible = func_ty.params.get(1..).unwrap_or(&[]);
                self.check_method_call_args(method_name, &func_ty.type_params, visible, args, span);
                Some(*func_ty.return_type.clone())
            }
            MethodKind::Procedure(proc_ty) => {
                let visible = proc_ty.params.get(1..).unwrap_or(&[]);
                self.check_method_call_args(method_name, &proc_ty.type_params, visible, args, span);
                if allow_procedure_result {
                    Some(Ty::Unit)
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Interface method `{}.{method_name}` is a procedure — it does not return a value", iface.name),
                        "Use a function method if you need a return value.",
                        span,
                    );
                    Some(Ty::Error)
                }
            }
        }
    }
}
