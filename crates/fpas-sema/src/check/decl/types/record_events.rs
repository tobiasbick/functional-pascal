//! Record event declaration checking.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::Checker;
use super::record_accessors::{AccessorMember, AccessorOwner};
use crate::types::{EventTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
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
        let owner_unit = self
            .scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.clone())
            .or_else(|| owner_unit_from_type_name(type_name));
        for event in events {
            if !self.register_record_member_name(type_name, &event.name, event.span, seen_members) {
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
            let member = AccessorMember {
                owner: AccessorOwner::Event,
                type_name,
                record_ty,
                name: &event.name,
                value_ty: &option_handler,
                span: event.span,
            };
            let getter = self.resolve_record_getter(&member, &event.read);
            let setter = self.resolve_record_setter(&member, &event.write);
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
}

/// Unit prefix of a linked type name, or `None` for program-local types.
pub(crate) fn owner_unit_from_type_name(type_name: &str) -> Option<String> {
    type_name.rsplit_once('.').map(|(unit, _)| unit.to_string())
}
