//! Codec validation outcomes map directly to the public native error catalog.

use crate::stdlib::{AggregateFactory, StdError};
use crate::{SourceLocation, Value};

use super::MAX_BYTES;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CodecFailure {
    InvalidByte { offset: usize, value: i64 },
    InvalidCharacter { offset: usize },
    InvalidLength { length: usize },
    InvalidPadding { offset: usize },
    NonCanonical { offset: usize },
    LimitExceeded,
}

impl CodecFailure {
    pub fn into_value(
        self,
        factory: &dyn AggregateFactory,
        location: SourceLocation,
    ) -> Result<Value, StdError> {
        use crate::stdlib::native_errors::{NativeErrorFamily, value};
        let (variant, fields) = match self {
            Self::InvalidByte { offset, value } => {
                (0, vec![integer(offset), Value::Integer(value)])
            }
            Self::InvalidCharacter { offset } => (1, vec![integer(offset)]),
            Self::InvalidLength { length } => (2, vec![integer(length)]),
            Self::InvalidPadding { offset } => (3, vec![integer(offset)]),
            Self::NonCanonical { offset } => (4, vec![integer(offset)]),
            Self::LimitExceeded => (5, vec![integer(MAX_BYTES)]),
        };
        value(
            factory,
            NativeErrorFamily::Encoding,
            variant,
            fields,
            location,
        )
    }
}

fn integer(value: usize) -> Value {
    // All stored offsets and lengths have already passed the codec size bounds.
    Value::Integer(value as i64)
}
