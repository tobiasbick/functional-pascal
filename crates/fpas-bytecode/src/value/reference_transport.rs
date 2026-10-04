//! Detect call-bound storage authority in values that would retain or transport it.

use super::Value;

impl Value {
    /// Whether this value directly contains a synchronous reference or captures one.
    ///
    /// Inspect compound value data and callable captures iteratively. Capture cells are
    /// storage roots rather than value containers; their writes reject reference payloads.
    #[must_use]
    pub fn contains_reference(&self) -> bool {
        match self {
            Self::Reference(_) => return true,
            Self::Array(_)
            | Self::Dict(_)
            | Self::Record(_)
            | Self::Enum(_)
            | Self::ResultOk(_)
            | Self::ResultError(_)
            | Self::OptionSome(_)
            | Self::Function(_) => {}
            _ => return false,
        }
        let mut pending = vec![self];
        while let Some(value) = pending.pop() {
            match value {
                Self::Reference(_) => return true,
                Self::Array(values) => pending.extend(values.iter()),
                Self::Dict(entries) => {
                    for (key, value) in entries.iter() {
                        pending.push(key);
                        pending.push(value);
                    }
                }
                Self::Record(record) => pending.extend(record.body().values.iter()),
                Self::Enum(enumeration) => pending.extend(enumeration.body().values.iter()),
                Self::ResultOk(value) | Self::ResultError(value) | Self::OptionSome(value) => {
                    pending.push(value)
                }
                Self::Function(function) => {
                    pending.extend(function.captures.iter());
                }
                _ => {}
            }
        }
        false
    }
}
