//! Shared `read` / `write` accessor resolution for record properties and events.
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`,
//! `docs/pascal/language/types/record-events.md`

use super::Checker;
use crate::types::{MethodKind, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;

/// Record member kind whose accessors are being resolved.
#[derive(Clone, Copy)]
pub(super) enum AccessorOwner {
    /// `property Name: Type read Getter write Setter;`
    Property,
    /// `event Name: Handler read Getter write Setter;`
    Event,
}

impl AccessorOwner {
    fn label(self) -> &'static str {
        match self {
            Self::Property => "Property",
            Self::Event => "Event",
        }
    }

    fn lower_label(self) -> &'static str {
        match self {
            Self::Property => "property",
            Self::Event => "event",
        }
    }

    /// Accessor value type as written in signature hints.
    fn value_type(self) -> &'static str {
        match self {
            Self::Property => "PropertyType",
            Self::Event => "Option of Handler",
        }
    }

    /// Accessor value type placeholder in "declare this accessor" hints.
    fn value_placeholder(self) -> &'static str {
        match self {
            Self::Property => "…",
            Self::Event => "Option of …",
        }
    }

    fn generic_getter_hint(self) -> &'static str {
        match self {
            Self::Property => {
                "Use a non-generic instance function whose result matches the property type."
            }
            Self::Event => "Use a non-generic instance function returning `Option of Handler`.",
        }
    }

    fn generic_setter_hint(self) -> &'static str {
        match self {
            Self::Property => {
                "Use a non-generic instance procedure whose value parameter matches the property type."
            }
            Self::Event => "Use a non-generic instance procedure accepting `Option of Handler`.",
        }
    }

    fn getter_type_mismatch(self, qualified: &str, member_name: &str) -> (String, &'static str) {
        match self {
            Self::Property => (
                format!(
                    "Property getter `{qualified}` return type does not match property `{member_name}`"
                ),
                "The getter result type must match the property type.",
            ),
            Self::Event => (
                format!("Event getter `{qualified}` must return `Option of` the handler type"),
                "Declare `function Getter(Self: Record): Option of HandlerType`.",
            ),
        }
    }

    fn setter_type_mismatch(self, qualified: &str, member_name: &str) -> (String, &'static str) {
        match self {
            Self::Property => (
                format!(
                    "Property setter `{qualified}` value type does not match property `{member_name}`"
                ),
                "The setter value parameter type must match the property type.",
            ),
            Self::Event => (
                format!(
                    "Event setter `{qualified}` value type must be `Option of` the handler type"
                ),
                "Declare `procedure Setter(Self: Record; Value: Option of HandlerType)`.",
            ),
        }
    }
}

/// Property or event declaration whose accessor is being resolved.
pub(super) struct AccessorMember<'a> {
    /// Property or event.
    pub(super) owner: AccessorOwner,
    /// Declaring record type name.
    pub(super) type_name: &'a str,
    /// Declaring record type.
    pub(super) record_ty: &'a Ty,
    /// Declared member name.
    pub(super) name: &'a str,
    /// Value type the accessors read and write.
    pub(super) value_ty: &'a Ty,
    /// Source span of the member declaration.
    pub(super) span: Span,
}

impl Checker {
    /// Resolve a `read` accessor to its qualified instance function name.
    pub(super) fn resolve_record_getter(
        &mut self,
        member: &AccessorMember<'_>,
        getter_name: &str,
    ) -> Option<String> {
        let (qualified, method_kind) = self.resolve_record_accessor(member, getter_name, "read")?;
        let owner = member.owner;
        let label = owner.label();
        let value_type = owner.value_type();

        let func_ty = match method_kind {
            MethodKind::Function(func_ty) => func_ty,
            MethodKind::Procedure(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "{label} `{}.{}` `read` accessor `{getter_name}` must be a function",
                        member.type_name, member.name
                    ),
                    format!("Use `function Getter(Self: Record): {value_type}`, not a procedure."),
                    member.span,
                );
                return None;
            }
        };

        if !func_ty.type_params.is_empty() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("{label} getter `{qualified}` cannot be generic"),
                owner.generic_getter_hint(),
                member.span,
            );
            return None;
        }
        if func_ty.params.len() != 1 {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("{label} getter `{qualified}` must take only `Self`"),
                format!(
                    "Declare `function Getter(Self: Record): {value_type}` with no extra parameters."
                ),
                member.span,
            );
            return None;
        }
        if func_ty.params[0].mutable {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("{label} getter `{qualified}` cannot take `mutable Self`"),
                "Declare the getter with `Self` passed by value.",
                member.span,
            );
            return None;
        }
        if !member
            .value_ty
            .compatible_with(func_ty.return_type.as_ref())
        {
            let (message, hint) = owner.getter_type_mismatch(&qualified, member.name);
            self.error_with_code(SEMA_TYPE_MISMATCH, message, hint, member.span);
            return None;
        }
        Some(qualified)
    }

    /// Resolve a `write` accessor to its qualified instance procedure name.
    pub(super) fn resolve_record_setter(
        &mut self,
        member: &AccessorMember<'_>,
        setter_name: &str,
    ) -> Option<String> {
        let (qualified, method_kind) =
            self.resolve_record_accessor(member, setter_name, "write")?;
        let owner = member.owner;
        let label = owner.label();
        let value_type = owner.value_type();

        let proc_ty = match method_kind {
            MethodKind::Procedure(proc_ty) => proc_ty,
            MethodKind::Function(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "{label} `{}.{}` `write` accessor `{setter_name}` must be a procedure",
                        member.type_name, member.name
                    ),
                    format!(
                        "Use `procedure Setter(Self: Record; Value: {value_type})`, not a function."
                    ),
                    member.span,
                );
                return None;
            }
        };

        if !proc_ty.type_params.is_empty() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("{label} setter `{qualified}` cannot be generic"),
                owner.generic_setter_hint(),
                member.span,
            );
            return None;
        }
        if proc_ty.params.len() != 2 {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("{label} setter `{qualified}` must take `Self` and one value parameter"),
                format!("Declare `procedure Setter(Self: Record; Value: {value_type})`."),
                member.span,
            );
            return None;
        }
        if proc_ty.params[0].mutable || proc_ty.params[1].mutable {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("{label} setter `{qualified}` must take `Self` and its value by value"),
                format!(
                    "Remove `mutable` from both {} setter parameters.",
                    owner.lower_label()
                ),
                member.span,
            );
            return None;
        }
        if !member.value_ty.compatible_with(&proc_ty.params[1].ty) {
            let (message, hint) = owner.setter_type_mismatch(&qualified, member.name);
            self.error_with_code(SEMA_TYPE_MISMATCH, message, hint, member.span);
            return None;
        }
        Some(qualified)
    }

    /// Reject static accessors and look up the instance method named by an accessor.
    fn resolve_record_accessor(
        &mut self,
        member: &AccessorMember<'_>,
        accessor_name: &str,
        role: &str,
    ) -> Option<(String, MethodKind)> {
        let Ty::Record(record) = member.record_ty else {
            return None;
        };
        let type_name = member.type_name;
        let label = member.owner.label();
        let value_type = member.owner.value_type();
        let qualified = format!("{type_name}.{accessor_name}");

        if record
            .static_functions
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(accessor_name))
            || record
                .static_procedures
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(accessor_name))
        {
            let hint = if role == "read" {
                format!(
                    "Use an instance function with signature `function Getter(Self: Record): {value_type}`."
                )
            } else {
                format!(
                    "Use an instance procedure with signature `procedure Setter(Self: Record; Value: {value_type})`."
                )
            };
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "{label} `{type_name}.{}` cannot use static routine `{accessor_name}` as `{role}`",
                    member.name
                ),
                hint,
                member.span,
            );
            return None;
        }

        let Some(method_kind) = self.resolve_method_kind(record, accessor_name, &qualified) else {
            let placeholder = member.owner.value_placeholder();
            let hint = if role == "read" {
                format!(
                    "Declare `function {accessor_name}(Self: {type_name}): {placeholder}` on the same record."
                )
            } else {
                format!(
                    "Declare `procedure {accessor_name}(Self: {type_name}; Value: {placeholder})` on the same record."
                )
            };
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!(
                    "{label} `{type_name}.{}` references unknown `{role}` accessor `{accessor_name}`",
                    member.name
                ),
                hint,
                member.span,
            );
            return None;
        };

        Some((qualified, method_kind))
    }
}
