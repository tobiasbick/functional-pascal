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
    /// Property and bound-method metadata use separate keys because properties may appear in the
    /// middle of a designator while a bound method must be its final member.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    pub(crate) fn check_record_member_access(
        &mut self,
        ty: &Ty,
        member: &str,
        span: Span,
        property_key: Option<(usize, usize)>,
        bound_key: Option<(usize, usize)>,
    ) -> Ty {
        match ty {
            Ty::Record(record_ty) => {
                if self.reject_private_record_member(record_ty, member, span) {
                    return Ty::Error;
                }
                if let Some((_, field_ty)) = record_ty
                    .fields
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(member))
                {
                    return field_ty.clone();
                }

                if let Some(property) = self.find_record_property(record_ty, member) {
                    return self.property_read_type(
                        &record_ty.name,
                        member,
                        &property,
                        span,
                        property_key,
                    );
                }

                if self.find_record_event_on_type(record_ty, member).is_some() {
                    return self.reject_bare_event_read(&record_ty.name, member, span);
                }

                if let Some(routine_kind) = self.static_routine_kind_on_record(record_ty, member) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "`{member}` is a static {routine_kind} and cannot be bound from a value"
                        ),
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
                            "Record `{}` has no field, property, event, or method `{member}`",
                            record_ty.name
                        ),
                        "Check the field, property, event, or instance method name against the record type.",
                        span,
                    );
                    return Ty::Error;
                };

                self.bound_callable_type(&qualified, &method_kind, span, bound_key)
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("`.{member}` requires a record value"),
                    "Only records support field, property, event, and bound-method access with `.`.",
                    span,
                );
                Ty::Error
            }
        }
    }

    fn property_read_type(
        &mut self,
        record_name: &str,
        member: &str,
        property: &crate::types::PropertyTy,
        span: Span,
        bind_key: Option<(usize, usize)>,
    ) -> Ty {
        let Some(getter) = &property.getter else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Property `{record_name}.{member}` is write-only"),
                format!("Assign to `{member}` instead of reading it."),
                span,
            );
            return Ty::Error;
        };
        if let Some((key, receiver_part_count)) = bind_key {
            self.property_reads.entry(key).or_default().push(
                super::super::context::PropertyReadInfo {
                    getter_name: getter.clone(),
                    receiver_part_count,
                },
            );
        }
        property.ty.clone()
    }

    /// Look up a property on `record_ty`, falling back to the canonical type symbol.
    ///
    /// Method return types may embed a `RecordTy` snapshot taken before properties were
    /// attached; the scope entry holds the complete type.
    fn find_record_property(
        &self,
        record_ty: &crate::types::RecordTy,
        member: &str,
    ) -> Option<crate::types::PropertyTy> {
        if let Some((_, property)) = record_ty
            .properties
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(member))
        {
            return Some(property.clone());
        }
        if record_ty.name == "<anonymous>" {
            return None;
        }
        let symbol = self.scopes.lookup(&record_ty.name)?;
        let Ty::Record(canonical) = &symbol.ty else {
            return None;
        };
        canonical
            .properties
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(member))
            .map(|(_, property)| property.clone())
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
