//! Unit-owned enum identities in exported static record fields.
//! See `docs/pascal/language/basics/constants.md`.

use fpas_unit::interface::{ConstantValue, RecordConstant, RecordConstantField};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Qualify owned enum values without duplicating shared nested-record graphs.
/// See `docs/pascal/language/basics/constants.md`.
pub(super) fn qualify(
    record: &Arc<RecordConstant>,
    unit_name: &str,
    own_types: &HashSet<String>,
) -> Arc<RecordConstant> {
    fn visit(
        record: &Arc<RecordConstant>,
        unit_name: &str,
        own_types: &HashSet<String>,
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
                        if let ConstantValue::EnumValue { enum_name, .. } = &mut value {
                            *enum_name = super::qualify_owned_name(enum_name, unit_name, own_types);
                        }
                        RecordConstantField::Scalar(value)
                    }
                    RecordConstantField::Record(record) => {
                        RecordConstantField::Record(visit(record, unit_name, own_types, seen))
                    }
                };
                (name.clone(), value)
            })
            .collect();
        let value = Arc::new(RecordConstant { fields });
        seen.insert(key, Arc::clone(&value));
        value
    }
    visit(record, unit_name, own_types, &mut HashMap::new())
}
