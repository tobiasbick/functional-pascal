//! Record event declaration checking.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::Checker;
use crate::scope::canonical_symbol_name;
use crate::types::{EventTy, MethodKind, Ty};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::RecordEvent;
use std::collections::HashSet;

impl Checker {
    /// Validate event declarations and resolve `Option of Handler` accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(super) fn check_record_events(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        events: &[RecordEvent],
        seen_members: &mut HashSet<String>,
    ) -> Vec<(String, EventTy)> {
        let mut checked = Vec::new();
        let owner_unit = owner_unit_from_type_name(type_name);
        for event in events {
            if !seen_members.insert(canonical_symbol_name(&event.name)) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate record member `{type_name}.{}`", event.name),
                    "Each field, method, static function, property, and event name must be unique within the record type.",
                    event.span,
                );
                continue;
            }

            if event.read.is_empty() || event.write.is_empty() {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Event `{type_name}.{}` must declare both `read` and `write`",
                        event.name
                    ),
                    "Write `event Name: HandlerType read Getter write Setter;`.",
                    event.span,
                );
                continue;
            }

            let handler_ty = self.resolve_type_expr(&event.type_expr);
            if !matches!(handler_ty, Ty::Function(_) | Ty::Procedure(_) | Ty::Error) {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Event `{type_name}.{}` handler type must be a function or procedure type",
                        event.name
                    ),
                    "Declare the event as `event Name: procedure(...)` or `event Name: function(...): T`.",
                    event.span,
                );
                continue;
            }

            let option_handler = Ty::Option(Box::new(handler_ty.clone()));
            let getter = self.resolve_event_getter(
                type_name,
                record_ty,
                &event.read,
                &option_handler,
                event,
            );
            let setter = self.resolve_event_setter(
                type_name,
                record_ty,
                &event.write,
                &option_handler,
                event,
            );
            let (Some(getter), Some(setter)) = (getter, setter) else {
                continue;
            };

            checked.push((
                event.name.clone(),
                EventTy {
                    handler_ty,
                    getter,
                    setter,
                    owner_unit: owner_unit.clone(),
                },
            ));
        }
        checked
    }

    fn resolve_event_getter(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        getter_name: &str,
        option_handler: &Ty,
        event: &RecordEvent,
    ) -> Option<String> {
        let Ty::Record(record) = record_ty else {
            return None;
        };
        let qualified = format!("{type_name}.{getter_name}");

        if record
            .static_functions
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(getter_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Event `{type_name}.{}` cannot use static function `{getter_name}` as `read`",
                    event.name
                ),
                "Use an instance function with signature `function Getter(Self: Record): Option of Handler`.",
                event.span,
            );
            return None;
        }

        let Some(method_kind) = self.resolve_method_kind(record, getter_name, &qualified) else {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!(
                    "Event `{type_name}.{}` references unknown `read` accessor `{getter_name}`",
                    event.name
                ),
                format!(
                    "Declare `function {getter_name}(Self: {type_name}): Option of …` on the same record."
                ),
                event.span,
            );
            return None;
        };

        match method_kind {
            MethodKind::Function(func_ty) => {
                if !func_ty.type_params.is_empty() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Event getter `{qualified}` cannot be generic"),
                        "Use a non-generic instance function returning `Option of Handler`.",
                        event.span,
                    );
                    return None;
                }
                if func_ty.params.len() != 1 {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Event getter `{qualified}` must take only `Self`"),
                        "Declare `function Getter(Self: Record): Option of Handler` with no extra parameters.",
                        event.span,
                    );
                    return None;
                }
                if func_ty.params[0].mutable {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Event getter `{qualified}` cannot take `mutable Self`"),
                        "Declare the getter with `Self` passed by value.",
                        event.span,
                    );
                    return None;
                }
                if !option_handler.compatible_with(func_ty.return_type.as_ref()) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Event getter `{qualified}` must return `Option of` the handler type"
                        ),
                        "Declare `function Getter(Self: Record): Option of HandlerType`.",
                        event.span,
                    );
                    return None;
                }
                Some(qualified)
            }
            MethodKind::Procedure(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Event `{type_name}.{}` `read` accessor `{getter_name}` must be a function",
                        event.name
                    ),
                    "Use `function Getter(Self: Record): Option of Handler`, not a procedure.",
                    event.span,
                );
                None
            }
        }
    }

    fn resolve_event_setter(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        setter_name: &str,
        option_handler: &Ty,
        event: &RecordEvent,
    ) -> Option<String> {
        let Ty::Record(record) = record_ty else {
            return None;
        };
        let qualified = format!("{type_name}.{setter_name}");

        if record
            .static_functions
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(setter_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Event `{type_name}.{}` cannot use static function `{setter_name}` as `write`",
                    event.name
                ),
                "Use an instance procedure with signature `procedure Setter(Self: Record; Value: Option of Handler)`.",
                event.span,
            );
            return None;
        }

        let Some(method_kind) = self.resolve_method_kind(record, setter_name, &qualified) else {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!(
                    "Event `{type_name}.{}` references unknown `write` accessor `{setter_name}`",
                    event.name
                ),
                format!(
                    "Declare `procedure {setter_name}(Self: {type_name}; Value: Option of …)` on the same record."
                ),
                event.span,
            );
            return None;
        };

        match method_kind {
            MethodKind::Procedure(proc_ty) => {
                if !proc_ty.type_params.is_empty() {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Event setter `{qualified}` cannot be generic"),
                        "Use a non-generic instance procedure accepting `Option of Handler`.",
                        event.span,
                    );
                    return None;
                }
                if proc_ty.params.len() != 2 {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Event setter `{qualified}` must take `Self` and one value parameter"
                        ),
                        "Declare `procedure Setter(Self: Record; Value: Option of Handler)`.",
                        event.span,
                    );
                    return None;
                }
                if proc_ty.params[0].mutable || proc_ty.params[1].mutable {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Event setter `{qualified}` must take `Self` and its value by value"
                        ),
                        "Remove `mutable` from both event setter parameters.",
                        event.span,
                    );
                    return None;
                }
                if !option_handler.compatible_with(&proc_ty.params[1].ty) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!(
                            "Event setter `{qualified}` value type must be `Option of` the handler type"
                        ),
                        "Declare `procedure Setter(Self: Record; Value: Option of HandlerType)`.",
                        event.span,
                    );
                    return None;
                }
                Some(qualified)
            }
            MethodKind::Function(_) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Event `{type_name}.{}` `write` accessor `{setter_name}` must be a procedure",
                        event.name
                    ),
                    "Use `procedure Setter(Self: Record; Value: Option of Handler)`, not a function.",
                    event.span,
                );
                None
            }
        }
    }
}

/// Unit prefix of a linked type name, or `None` for program-local types.
pub(crate) fn owner_unit_from_type_name(type_name: &str) -> Option<String> {
    type_name.rsplit_once('.').map(|(unit, _)| unit.to_string())
}
