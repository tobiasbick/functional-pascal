//! Record fields and ownership collected without evaluating defaults or bodies.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::Checker;
use crate::types::RecordTy;
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{RecordType, TypeDef, Visibility};
use std::collections::HashSet;

impl Checker {
    /// Resolve stored fields and preserve unit ownership and private-member metadata.
    pub(super) fn collect_record_fields(
        &mut self,
        definition: &TypeDef,
        record: &RecordType,
    ) -> RecordTy {
        let mut seen = HashSet::new();
        let mut fields = Vec::new();
        for field in &record.fields {
            if !seen.insert(field.name.to_ascii_lowercase()) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate record member `{}`", field.name),
                    "Each field, method, and static routine name must be unique within the record type.",
                    field.span,
                );
                continue;
            }
            fields.push((field.name.clone(), self.resolve_type_expr(&field.type_expr)));
        }
        let owner_unit = self
            .scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.clone());
        let private_members = if owner_unit.is_some() {
            record
                .fields
                .iter()
                .filter(|field| field.visibility == Visibility::Private)
                .map(|field| field.name.clone())
                .chain(
                    record
                        .methods
                        .iter()
                        .filter(|method| method.visibility() == Visibility::Private)
                        .map(|method| method.name().to_string()),
                )
                .collect()
        } else {
            Vec::new()
        };
        RecordTy {
            name: definition.name.clone(),
            type_params: Self::resolve_type_params(&definition.type_params),
            type_args: Vec::new(),
            owner_unit,
            private_members,
            fields,
            methods: Vec::new(),
            static_functions: Vec::new(),
            static_procedures: Vec::new(),
        }
    }
}
