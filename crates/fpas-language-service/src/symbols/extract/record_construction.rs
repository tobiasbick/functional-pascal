//! Declaration-order constructor signatures, retaining field defaults for editor help.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::type_text;
use crate::{CallableSignature, DocumentSnapshot};
use fpas_parser::{TypeBody, TypeDef};

/// Describe named field construction only for concrete record declarations.
pub(super) fn constructor_signature(
    snapshot: &DocumentSnapshot,
    definition: &TypeDef,
) -> Option<CallableSignature> {
    let TypeBody::Record(record) = &definition.body else {
        return None;
    };
    let parameters = record
        .fields
        .iter()
        .map(|field| {
            let mut label = format!("{}: {}", field.name, type_text(snapshot, &field.type_expr));
            if let Some(value) = &field.default_value {
                let span = value.span();
                if let Some(text) = snapshot
                    .source()
                    .get(span.offset..span.offset.saturating_add(span.length))
                {
                    label.push_str(&format!(" := {}", text.trim()));
                }
            }
            label
        })
        .collect::<Vec<_>>();
    Some(CallableSignature {
        label: format!(
            "{}({}): {}",
            definition.name,
            parameters.join("; "),
            definition.name
        ),
        parameters,
    })
}
