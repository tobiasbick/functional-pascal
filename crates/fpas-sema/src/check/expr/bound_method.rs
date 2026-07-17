//! Bound record method values (`C.Add` without a call).
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use super::super::Checker;
use crate::types::{FunctionTy, MethodKind, ProcedureTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;

use super::super::context::BoundMethodInfo;

impl Checker {
    /// Resolve a bare record member as a field or as a bound instance-method value.
    ///
    /// When `bind_key` is `Some((key, receiver_part_count))`, a successful method resolution
    /// records metadata for codegen.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    pub(crate) fn check_record_member_access(
        &mut self,
        ty: &Ty,
        member: &str,
        span: Span,
        bind_key: Option<(usize, usize)>,
    ) -> Ty {
        match ty {
            Ty::Record(record_ty) => {
                if let Some((_, field_ty)) = record_ty
                    .fields
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(member))
                {
                    return field_ty.clone();
                }

                if self.is_static_function_on_record(record_ty, member) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("`{member}` is a static function and cannot be bound from a value"),
                        format!(
                            "Call it through the type as `{}.{}`, or use an instance method.",
                            record_ty.name, member
                        ),
                        span,
                    );
                    return Ty::Error;
                }

                let qualified = format!("{}.{}", record_ty.name, member);
                let Some(method_kind) = self.resolve_method_kind(record_ty, member, &qualified)
                else {
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        format!(
                            "Record `{}` has no field or method `{member}`",
                            record_ty.name
                        ),
                        "Check the field or instance method name against the record type.",
                        span,
                    );
                    return Ty::Error;
                };

                self.bound_callable_type(&qualified, &method_kind, span, bind_key)
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("`.{member}` requires a record value"),
                    "Only records support field and bound-method access with `.`.",
                    span,
                );
                Ty::Error
            }
        }
    }

    fn bound_callable_type(
        &mut self,
        qualified: &str,
        method_kind: &MethodKind,
        span: Span,
        bind_key: Option<(usize, usize)>,
    ) -> Ty {
        match method_kind {
            MethodKind::Function(func_ty) => {
                let Some(self_param) = func_ty.params.first() else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Record method `{qualified}` must declare `Self` as its first parameter"
                        ),
                        "Declare the method as `function Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return Ty::Error;
                };
                if self_param.mutable {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Cannot bind method `{qualified}` because `Self` is `mutable`"
                        ),
                        "Capture a `mutable var` in a closure instead of binding a mutable receiver.",
                        span,
                    );
                    return Ty::Error;
                }
                let visible = func_ty.params[1..].to_vec();
                if let Some((key, receiver_part_count)) = bind_key {
                    let Ok(visible_arity) = u8::try_from(visible.len()) else {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            format!("Method `{qualified}` has too many parameters to bind"),
                            "Reduce the number of explicit parameters.",
                            span,
                        );
                        return Ty::Error;
                    };
                    self.bound_methods.insert(
                        key,
                        BoundMethodInfo {
                            qualified_name: qualified.to_string(),
                            visible_arity,
                            receiver_part_count,
                        },
                    );
                }
                Ty::Function(FunctionTy {
                    type_params: func_ty.type_params.clone(),
                    params: visible,
                    return_type: func_ty.return_type.clone(),
                    variadic: func_ty.variadic,
                })
            }
            MethodKind::Procedure(proc_ty) => {
                let Some(self_param) = proc_ty.params.first() else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Record method `{qualified}` must declare `Self` as its first parameter"
                        ),
                        "Declare the method as `procedure Name(Self: RecordType; ...)`.",
                        span,
                    );
                    return Ty::Error;
                };
                if self_param.mutable {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Cannot bind method `{qualified}` because `Self` is `mutable`"
                        ),
                        "Capture a `mutable var` in a closure instead of binding a mutable receiver.",
                        span,
                    );
                    return Ty::Error;
                }
                let visible = proc_ty.params[1..].to_vec();
                if let Some((key, receiver_part_count)) = bind_key {
                    let Ok(visible_arity) = u8::try_from(visible.len()) else {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            format!("Method `{qualified}` has too many parameters to bind"),
                            "Reduce the number of explicit parameters.",
                            span,
                        );
                        return Ty::Error;
                    };
                    self.bound_methods.insert(
                        key,
                        BoundMethodInfo {
                            qualified_name: qualified.to_string(),
                            visible_arity,
                            receiver_part_count,
                        },
                    );
                }
                Ty::Procedure(ProcedureTy {
                    type_params: proc_ty.type_params.clone(),
                    params: visible,
                    variadic: proc_ty.variadic,
                })
            }
        }
    }
}
