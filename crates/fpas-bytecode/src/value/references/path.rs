//! Checked record and collection projections for synchronous storage references.

use super::ReferenceError;
use crate::{RecordTypeId, Value};

#[cfg(test)]
mod tests;

/// One already evaluated segment of a writable storage path.
#[derive(Debug, Clone)]
pub enum ReferenceStep {
    /// A stored field in the expected executable-local record layout.
    Field {
        /// Record layout required by the compiled projection.
        record: RecordTypeId,
        /// Field slot in layout order.
        field: usize,
    },
    /// An evaluated array index or dictionary key, retained as a value snapshot.
    Index(Value),
}

/// An authority, layout, or bounds check failed for a selected storage path.
#[derive(Debug, Clone, PartialEq)]
pub enum ReferencePathError {
    /// The storage root is reserved, suspended, exclusive, or released.
    Access(ReferenceError),
    /// A projection or array index has the wrong runtime value kind.
    TypeMismatch {
        /// Runtime kind required by this projection.
        expected: &'static str,
        /// Actual runtime kind.
        actual: &'static str,
    },
    /// A stored record does not have the projection's compiled layout.
    RecordLayout {
        /// Expected executable-local record identity.
        expected: RecordTypeId,
        /// Actual executable-local record identity.
        actual: RecordTypeId,
    },
    /// A compiled field slot does not exist in the runtime record body.
    FieldBounds {
        /// Requested field slot.
        field: usize,
        /// Number of stored fields.
        count: usize,
    },
    /// An array index is negative, unrepresentable, or outside the current bounds.
    ArrayBounds {
        /// Evaluated signed index, without narrowing.
        index: i64,
        /// Current number of elements.
        length: usize,
    },
    /// A selected dictionary element does not exist.
    MissingKey(Value),
}

impl std::fmt::Display for ReferencePathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Access(error) => error.fmt(formatter),
            Self::TypeMismatch { expected, actual } => {
                write!(formatter, "Storage path requires {expected}, got {actual}")
            }
            Self::RecordLayout { expected, actual } => write!(
                formatter,
                "Storage path requires record layout {}, got {}",
                expected.get(),
                actual.get()
            ),
            Self::FieldBounds { field, count } => {
                write!(
                    formatter,
                    "Record field slot {field} out of bounds (fields {count})"
                )
            }
            Self::ArrayBounds { index, length } => {
                write!(
                    formatter,
                    "Array index {index} out of bounds (len {length})"
                )
            }
            Self::MissingKey(key) => write!(formatter, "Key `{key}` not found in dict"),
        }
    }
}

impl std::error::Error for ReferencePathError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Access(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ReferenceError> for ReferencePathError {
    fn from(error: ReferenceError) -> Self {
        Self::Access(error)
    }
}

pub(super) fn read<'a>(
    root: &'a Value,
    steps: &[ReferenceStep],
) -> Result<&'a Value, ReferencePathError> {
    let mut selected = root;
    for step in steps {
        selected = step.read(selected)?;
    }
    Ok(selected)
}

// Keep replaced values in the caller's slots so their destructors run after storage unlocks.
pub(super) fn replace(
    root: &mut Value,
    steps: &[ReferenceStep],
    replacement: &mut Value,
) -> Result<(), ReferencePathError> {
    let mut selected = root;
    for step in steps {
        selected = step.write_target(selected)?;
    }
    std::mem::swap(selected, replacement);
    Ok(())
}

impl ReferenceStep {
    fn read<'a>(&self, value: &'a Value) -> Result<&'a Value, ReferencePathError> {
        let position = self.position(value)?;
        match value {
            Value::Record(stored) => Ok(&stored.body().values[position]),
            Value::Array(values) => Ok(&values[position]),
            Value::Dict(entries) => Ok(&entries[position].1),
            _ => Err(self.type_error(value)),
        }
    }

    fn write_target<'a>(&self, value: &'a mut Value) -> Result<&'a mut Value, ReferencePathError> {
        let position = self.position(value)?;
        match value {
            Value::Record(stored) => Ok(&mut stored.values_mut()[position]),
            Value::Array(values) => Ok(&mut values[position]),
            Value::Dict(entries) => Ok(&mut entries[position].1),
            _ => Err(self.type_error(value)),
        }
    }

    fn position(&self, value: &Value) -> Result<usize, ReferencePathError> {
        match (self, value) {
            (Self::Field { record, field }, Value::Record(stored)) => {
                let body = stored.body();
                check_record(*record, body.layout.record)?;
                if *field < body.values.len() {
                    Ok(*field)
                } else {
                    Err(ReferencePathError::FieldBounds {
                        field: *field,
                        count: body.values.len(),
                    })
                }
            }
            (Self::Index(index), Value::Array(values)) => array_index(index, values.len()),
            (Self::Index(key), Value::Dict(entries)) => entries
                .iter()
                .position(|(candidate, _)| candidate.language_equal(key))
                .ok_or_else(|| ReferencePathError::MissingKey(key.clone())),
            _ => Err(self.type_error(value)),
        }
    }

    fn type_error(&self, value: &Value) -> ReferencePathError {
        ReferencePathError::TypeMismatch {
            expected: match self {
                Self::Field { .. } => "record",
                Self::Index(_) => "array or dictionary",
            },
            actual: value.type_name(),
        }
    }
}

fn check_record(expected: RecordTypeId, actual: RecordTypeId) -> Result<(), ReferencePathError> {
    if expected == actual {
        Ok(())
    } else {
        Err(ReferencePathError::RecordLayout { expected, actual })
    }
}

fn array_index(value: &Value, length: usize) -> Result<usize, ReferencePathError> {
    let Value::Integer(index) = value else {
        return Err(ReferencePathError::TypeMismatch {
            expected: "integer array index",
            actual: value.type_name(),
        });
    };
    usize::try_from(*index)
        .ok()
        .filter(|index| *index < length)
        .ok_or(ReferencePathError::ArrayBounds {
            index: *index,
            length,
        })
}
