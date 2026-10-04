//! Call-site generic inference and substitution.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::super::Checker;
use crate::types::{GenericParamDef, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use std::collections::HashMap;

impl Checker {
    /// Infer caller-supplied type arguments, enforce consistent reuse, and validate constraints.
    pub(crate) fn validate_routine_constraints(
        &mut self,
        type_params: &[GenericParamDef],
        params: &[crate::types::ParamTy],
        arg_types: &[Ty],
        span: Span,
    ) -> HashMap<String, Ty> {
        if type_params.is_empty() {
            return HashMap::new();
        }

        let mut inferred = HashMap::new();

        for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
            self.collect_type_param_bindings(&param.ty, arg_ty, &mut inferred, span);
        }

        // Build a Vec<GenericParamDef> + Vec<Ty> for only the params we inferred.
        let mut check_params = Vec::new();
        let mut check_args = Vec::new();
        for tp in type_params {
            if let Some(arg) = inferred.get(&tp.name.to_ascii_lowercase()) {
                check_params.push(tp.clone());
                check_args.push(arg.clone());
            }
        }

        if !check_params.is_empty() {
            self.validate_constraints(&check_params, &check_args, span);
        }

        inferred
    }

    /// Substitute inferred routine arguments using the shared type operation.
    pub(crate) fn substitute_type_params(ty: &Ty, inferred: &HashMap<String, Ty>) -> Ty {
        ty.substitute(inferred)
    }

    /// Extend consistent generic bindings from a declared and actual type pair.
    pub(crate) fn collect_type_param_bindings(
        &mut self,
        declared: &Ty,
        actual: &Ty,
        inferred: &mut HashMap<String, Ty>,

        span: Span,
    ) {
        let declared_visible = self.resolve_visible_type(declared);
        let actual_visible = self.resolve_visible_type(actual);

        match (&declared_visible, &actual_visible) {
            (Ty::GenericParam(name, _), actual_ty) if !actual_ty.is_error() => {
                let key = name.to_ascii_lowercase();
                if let Some(previous) = inferred.get(&key) {
                    if !previous.assignment_compatible_with(actual_ty)
                        || !actual_ty.assignment_compatible_with(previous)
                    {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            format!(
                                "Type parameter `{name}` was inferred as `{previous}`, but was also used with `{actual_ty}`"
                            ),
                            format!(
                                "Use the same concrete type for every argument bound to `{name}`."
                            ),
                            span,
                        );
                    } else {
                        inferred.insert(key, previous.complete_inference_with(actual_ty));
                    }
                } else {
                    inferred.insert(key, actual_ty.clone());
                }
            }
            (Ty::Array(declared_inner), Ty::Array(actual_inner)) => {
                self.collect_type_param_bindings(declared_inner, actual_inner, inferred, span);
            }
            (Ty::Channel(declared_inner), Ty::Channel(actual_inner)) => {
                self.collect_type_param_bindings(declared_inner, actual_inner, inferred, span);
            }
            (Ty::Dict(declared_key, declared_value), Ty::Dict(actual_key, actual_value)) => {
                self.collect_type_param_bindings(declared_key, actual_key, inferred, span);
                self.collect_type_param_bindings(declared_value, actual_value, inferred, span);
            }
            (Ty::Option(declared_inner), Ty::Option(actual_inner))
            | (Ty::Task(declared_inner), Ty::Task(actual_inner)) => {
                self.collect_type_param_bindings(declared_inner, actual_inner, inferred, span);
            }
            (Ty::Result(declared_ok, declared_err), Ty::Result(actual_ok, actual_err)) => {
                self.collect_type_param_bindings(declared_ok, actual_ok, inferred, span);
                self.collect_type_param_bindings(declared_err, actual_err, inferred, span);
            }
            (Ty::Record(declared), Ty::Record(actual))
                if declared.name.eq_ignore_ascii_case(&actual.name) =>
            {
                for (declared, actual) in declared.type_args.iter().zip(&actual.type_args) {
                    self.collect_type_param_bindings(declared, actual, inferred, span);
                }
            }
            (Ty::Enum(declared), Ty::Enum(actual))
                if declared.name.eq_ignore_ascii_case(&actual.name) =>
            {
                for (declared, actual) in declared.type_args.iter().zip(&actual.type_args) {
                    self.collect_type_param_bindings(declared, actual, inferred, span);
                }
            }
            (Ty::Function(declared_fn), Ty::Function(actual_fn)) => {
                for (declared_param, actual_param) in
                    declared_fn.params.iter().zip(actual_fn.params.iter())
                {
                    self.collect_type_param_bindings(
                        &declared_param.ty,
                        &actual_param.ty,
                        inferred,
                        span,
                    );
                }
                self.collect_type_param_bindings(
                    &declared_fn.return_type,
                    &actual_fn.return_type,
                    inferred,
                    span,
                );
            }
            (Ty::Procedure(declared_proc), Ty::Procedure(actual_proc)) => {
                for (declared_param, actual_param) in
                    declared_proc.params.iter().zip(actual_proc.params.iter())
                {
                    self.collect_type_param_bindings(
                        &declared_param.ty,
                        &actual_param.ty,
                        inferred,
                        span,
                    );
                }
            }
            _ => {}
        }
    }
}
