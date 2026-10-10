//! Dictionary key types must support equality.
//!
//! Key types are recorded while types resolve and validated after every declaration
//! is complete, so keys may name records and enums declared later in the unit.
//!
//! **Documentation:** `docs/pascal/language/types/dictionaries.md`

use super::super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use std::collections::HashSet;

/// One dictionary key type awaiting validation.
pub(crate) struct PendingKey {
    key: Ty,
    span: Span,
    literal: bool,
}

impl Checker {
    /// Records a dictionary key type written in a type at `span`.
    pub(crate) fn defer_dictionary_key_check(&mut self, key: Ty, span: Span) {
        self.pending_dictionary_keys.push(PendingKey {
            key,
            span,
            literal: false,
        });
    }

    /// Records the key type a dictionary literal infers from its first key.
    pub(crate) fn defer_dictionary_literal_key_check(&mut self, key: Ty, span: Span) {
        self.pending_dictionary_keys.push(PendingKey {
            key,
            span,
            literal: true,
        });
    }

    /// Reports each recorded key type whose values do not compare with `=`.
    ///
    /// A literal is reported only when no written type already reported the same key type.
    pub(crate) fn validate_dictionary_keys(&mut self) {
        let pending = std::mem::take(&mut self.pending_dictionary_keys);
        let invalid = pending
            .into_iter()
            .filter(|pending| !self.supports_equality(&pending.key))
            .collect::<Vec<_>>();
        let written = invalid
            .iter()
            .filter(|pending| !pending.literal)
            .map(|pending| pending.key.to_string())
            .collect::<HashSet<_>>();
        let mut reported = HashSet::new();
        for PendingKey { key, span, literal } in invalid {
            if (literal && written.contains(&key.to_string()))
                || !reported.insert((span.source_id, span.offset))
            {
                continue;
            }
            let hint = if matches!(key, Ty::GenericParam(_, None)) {
                "Add `: Comparable` to the type parameter, for example `function Index<K: Comparable>(...)`."
            } else {
                "Use a key type whose values compare with `=`: integer, real, boolean, string, an enum, a distinct type, or a record, enum, Option, or Result whose fields all compare."
            };
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Dictionary key type `{key}` does not support equality"),
                hint,
                span,
            );
        }
    }
}
