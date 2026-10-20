//! Resolve semantic Cause references into admitted executable-local IDs.
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

use super::strings::StringInterner;
use crate::LinkError;
use fpas_bytecode::{EnumTypeId, ErrorCause, ErrorInstanceKey, ErrorVariant};
use fpas_unit::object::ObjectErrorVariant;
use std::collections::BTreeMap;

pub(super) fn variant(
    error: &ObjectErrorVariant,
    families: &BTreeMap<ErrorInstanceKey, EnumTypeId>,
    strings: &mut StringInterner,
) -> Result<ErrorVariant, LinkError> {
    Ok(ErrorVariant {
        field_types: error.field_types.clone(),
        message: error
            .message
            .as_deref()
            .map(|message| strings.intern(message))
            .transpose()?,
        cause: error
            .cause
            .map(|field| {
                let key = error
                    .field_types
                    .get(usize::from(field))
                    .and_then(fpas_bytecode::TypeIdentity::error_key)
                    .ok_or_else(|| {
                        crate::errors::invalid("Cause has no concrete error field type")
                    })?;
                let family = families.get(&key).copied().ok_or_else(|| {
                    crate::errors::invalid(format!("missing Cause family `{}`", key.declaration))
                })?;
                Ok(ErrorCause { field, family })
            })
            .transpose()?,
    })
}
