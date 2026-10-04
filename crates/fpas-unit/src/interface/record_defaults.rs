//! Persisted record defaults and their implementation-only callable identities.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::ConstantValue;

/// A statically known scalar or a checked initializer in the declaring unit.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FieldDefault {
    /// Scalar retained for static expression evaluation in importing units.
    Constant(ConstantValue),
    /// Qualified implementation symbol; this is not a source callable value.
    Initializer {
        /// Qualified implementation symbol in the declaring unit.
        name: String,
        /// Nominal parameters whose concrete arguments must satisfy pure data rules.
        pure_parameters: Vec<super::GenericParameterId>,
    },
}

/// Derive an implementation symbol that cannot collide with source identifiers.
pub fn record_default_initializer(record: &str, owner: Option<&str>, field: &str) -> String {
    let record = match owner {
        Some(owner)
            if !record
                .to_ascii_lowercase()
                .starts_with(&format!("{}.", owner.to_ascii_lowercase())) =>
        {
            format!("{owner}.{record}")
        }
        _ => record.to_owned(),
    };
    format!("{record}.$default.{field}").to_ascii_lowercase()
}
