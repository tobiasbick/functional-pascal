//! Record property declaration checking.
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::{MethodKind, PropertyTy, Ty};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::RecordProperty;
use std::collections::HashSet;

impl Checker {
    /// Validate property declarations and resolve accessor qualified names.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(super) fn check_record_properties(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        properties: &[RecordProperty],
        seen_members: &mut HashSet<String>,
    ) -> Vec<(String, PropertyTy)> {
        let mut checked = Vec::new();
        for property in properties {
            if !seen_members.insert(canonical_symbol_name(&property.name)) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate record member `{type_name}.{}`", property.name),
                    "Each field, method, static routine, property, and event name must be unique within the record type.",
                    property.span,
                );
                continue;
            }

            if property.read.is_none() && property.write.is_none() {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Property `{type_name}.{}` must declare at least one of `read` or `write`",
                        property.name
                    ),
                    "Write `property Name: Type read Getter;`, `… write Setter;`, or both.",
                    property.span,
                );
                continue;
            }

            let prop_ty = self.resolve_type_expr(&property.type_expr);
            let getter = property.read.as_ref().and_then(|name| {
                self.resolve_property_getter(type_name, record_ty, name, &prop_ty, property)
            });
            let setter = property.write.as_ref().and_then(|name| {
                self.resolve_property_setter(type_name, record_ty, name, &prop_ty, property)
            });

            if property.read.is_some() && getter.is_none() {
                continue;
            }
            if property.write.is_some() && setter.is_none() {
                continue;
            }

            checked.push((
                property.name.clone(),
                PropertyTy {
                    ty: prop_ty,
                    getter,
                    setter,
                },
            ));
        }
        checked
    }

    fn resolve_property_getter(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        getter_name: &str,
        prop_ty: &Ty,
        property: &RecordProperty,
    ) -> Option<String> {
        let Ty::Record(record) = record_ty else {
            return None;
        };
        let qualified = format!("{type_name}.{getter_name}");

        if record
            .static_functions
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(getter_name))
            || record
                .static_procedures
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(getter_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Property `{type_name}.{}` cannot use static routine `{getter_name}` as `read`",
                    property.name
                ),
                "Use an instance function with signature `function Getter(Self: Record): PropertyType`.",
                property.span,
            );
            return None;
        }

        let Some(method_kind) = self.resolve_method_kind(record, getter_name, &qualified) else {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!(
                    "Property `{type_name}.{}` references unknown `read` accessor `{getter_name}`",
                    property.name
                ),
                format!(
                    "Declare `function {getter_name}(Self: {type_name}): …` on the same record."
                ),
                property.span,
            );
            return None;
        };

        match method_kind {
            MethodKind::Function(func_ty) => {
                if !func_ty.type_params.is_empty() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Property getter `{qualified}` cannot be generic"),
                        "Use a non-generic instance function whose result matches the property type.",
                        property.span,
                    );
                    return None;
                }
                if func_ty.params.len() != 1 {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Property getter `{qualified}` must take only `Self`"
                        ),
                        "Declare `function Getter(Self: Record): PropertyType` with no extra parameters.",
                        property.span,
                    );
                    return None;
                }
                if func_ty.params[0].mutable {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Property getter `{qualified}` cannot take `mutable Self`"),
                        "Declare the getter with `Self` passed by value.",
                        property.span,
                    );
                    return None;
                }
                if !prop_ty.compatible_with(func_ty.return_type.as_ref()) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Property getter `{qualified}` return type does not match property `{}`",
                            property.name
                        ),
                        "The getter result type must match the property type.",
                        property.span,
                    );
                    return None;
                }
                Some(qualified)
            }
            MethodKind::Procedure(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Property `{type_name}.{}` `read` accessor `{getter_name}` must be a function",
                        property.name
                    ),
                    "Use `function Getter(Self: Record): PropertyType`, not a procedure.",
                    property.span,
                );
                None
            }
        }
    }

    fn resolve_property_setter(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        setter_name: &str,
        prop_ty: &Ty,
        property: &RecordProperty,
    ) -> Option<String> {
        let Ty::Record(record) = record_ty else {
            return None;
        };
        let qualified = format!("{type_name}.{setter_name}");

        if record
            .static_functions
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(setter_name))
            || record
                .static_procedures
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(setter_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Property `{type_name}.{}` cannot use static routine `{setter_name}` as `write`",
                    property.name
                ),
                "Use an instance procedure with signature `procedure Setter(Self: Record; Value: PropertyType)`.",
                property.span,
            );
            return None;
        }

        let Some(method_kind) = self.resolve_method_kind(record, setter_name, &qualified) else {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!(
                    "Property `{type_name}.{}` references unknown `write` accessor `{setter_name}`",
                    property.name
                ),
                format!(
                    "Declare `procedure {setter_name}(Self: {type_name}; Value: …)` on the same record."
                ),
                property.span,
            );
            return None;
        };

        match method_kind {
            MethodKind::Procedure(proc_ty) => {
                if !proc_ty.type_params.is_empty() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Property setter `{qualified}` cannot be generic"),
                        "Use a non-generic instance procedure whose value parameter matches the property type.",
                        property.span,
                    );
                    return None;
                }
                if proc_ty.params.len() != 2 {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Property setter `{qualified}` must take `Self` and one value parameter"
                        ),
                        "Declare `procedure Setter(Self: Record; Value: PropertyType)`.",
                        property.span,
                    );
                    return None;
                }
                if proc_ty.params[0].mutable || proc_ty.params[1].mutable {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Property setter `{qualified}` must take `Self` and its value by value"
                        ),
                        "Remove `mutable` from both property setter parameters.",
                        property.span,
                    );
                    return None;
                }
                if !prop_ty.compatible_with(&proc_ty.params[1].ty) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Property setter `{qualified}` value type does not match property `{}`",
                            property.name
                        ),
                        "The setter value parameter type must match the property type.",
                        property.span,
                    );
                    return None;
                }
                Some(qualified)
            }
            MethodKind::Function(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Property `{type_name}.{}` `write` accessor `{setter_name}` must be a procedure",
                        property.name
                    ),
                    "Use `procedure Setter(Self: Record; Value: PropertyType)`, not a function.",
                    property.span,
                );
                None
            }
        }
    }
}
