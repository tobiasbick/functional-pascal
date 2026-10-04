//! Dictionary key admissibility uses the shared structural component contract.
//!
//! **Documentation:** `docs/pascal/language/types/dictionaries.md`.

use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

impl Checker {
    /// Validate a key after complete header resolution or concrete literal inference.
    pub(in crate::check) fn check_dictionary_key_type(&mut self, key: &Ty, span: Span) {
        if self.type_collection.defer_dictionary_key(key, span) || self.supports_equality(key) {
            return;
        }
        self.error_with_code(SEMA_TYPE_MISMATCH,
            format!("Unsupported dictionary key type `{key}`: keys must support structural equality"),
            "Use scalar or equatable value data; resources, tasks and callables cannot be keys. Constrain generic keys with `Equatable`.", span);
    }
}
