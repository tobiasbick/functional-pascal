//! Known record fields retained for constant projection across compiled units.
//! See `docs/pascal/language/basics/constants.md`.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use super::ConstantValue;

/// Known fields of a static record; its complete runtime value remains a unit global.
/// See `docs/pascal/language/pattern-matching/exhaustiveness.md`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecordConstant {
    /// Canonical field names and known scalar or nested-record values.
    /// Fields without a foldable representation are omitted.
    pub fields: BTreeMap<String, RecordConstantField>,
}

/// One known record field, sharing nested records instead of copying their value graphs.
/// See `docs/pascal/language/basics/constants.md`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecordConstantField {
    /// Foldable scalar value, including a simple enum member.
    Scalar(ConstantValue),
    /// Known projections of a nested record value.
    Record(Arc<RecordConstant>),
}

/// Normalize field and enum identities while retaining shared nested records.
/// See `docs/pascal/language/basics/constants.md`.
pub(super) fn canonicalize_record(record: &Arc<RecordConstant>) -> Arc<RecordConstant> {
    fn visit(
        record: &Arc<RecordConstant>,
        seen: &mut HashMap<usize, Arc<RecordConstant>>,
    ) -> Arc<RecordConstant> {
        let key = Arc::as_ptr(record) as usize;
        if let Some(value) = seen.get(&key) {
            return Arc::clone(value);
        }
        let fields = record
            .fields
            .iter()
            .map(|(name, field)| {
                let value = match field {
                    RecordConstantField::Scalar(value) => {
                        let mut value = value.clone();
                        super::symbols::canonicalize_constant(&mut value);
                        RecordConstantField::Scalar(value)
                    }
                    RecordConstantField::Record(record) => {
                        RecordConstantField::Record(visit(record, seen))
                    }
                };
                (name.to_ascii_lowercase(), value)
            })
            .collect();
        let value = Arc::new(RecordConstant { fields });
        seen.insert(key, Arc::clone(&value));
        value
    }
    visit(record, &mut HashMap::new())
}
