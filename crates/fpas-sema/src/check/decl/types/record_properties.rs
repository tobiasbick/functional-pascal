//! Record property declaration checking.
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use super::Checker;
use super::record_accessors::{AccessorMember, AccessorOwner};
use crate::types::{PropertyTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
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
            if !self.register_record_member_name(
                type_name,
                &property.name,
                property.span,
                seen_members,
            ) {
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
            let member = AccessorMember {
                owner: AccessorOwner::Property,
                type_name,
                record_ty,
                name: &property.name,
                value_ty: &prop_ty,
                span: property.span,
            };
            let getter = property
                .read
                .as_ref()
                .and_then(|name| self.resolve_record_getter(&member, name));
            let setter = property
                .write
                .as_ref()
                .and_then(|name| self.resolve_record_setter(&member, name));

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
}
