//! Declared `Std.Bytes.BufferError` variants.

use super::{ErrorFieldType::Integer, ErrorVariant};

pub(super) const VARIANTS: &[ErrorVariant] = &[
    ErrorVariant {
        name: "InvalidByte",
        message: "Byte value is outside 0..255",
        fields: &[("Offset", Integer), ("Value", Integer)],
    },
    ErrorVariant {
        name: "LimitExceeded",
        message: "Byte buffer limit exceeded",
        fields: &[("Limit", Integer)],
    },
    ErrorVariant {
        name: "AllocationFailed",
        message: "Byte buffer allocation failed",
        fields: &[("RequestedBytes", Integer)],
    },
];
