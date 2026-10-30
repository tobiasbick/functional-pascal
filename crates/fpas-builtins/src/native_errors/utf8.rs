//! Native UTF-8 error ABI shared with the source unit's public declarations.
use super::{
    ErrorFieldType::{Integer, String},
    ErrorVariant,
};

pub(super) const VARIANTS: &[ErrorVariant] = &[
    ErrorVariant {
        name: "InvalidByte",
        message: "Invalid UTF-8",
        fields: &[("Offset", Integer), ("Detail", String)],
    },
    ErrorVariant {
        name: "IncompleteSequence",
        message: "Invalid UTF-8",
        fields: &[("Offset", Integer)],
    },
    ErrorVariant {
        name: "AllocationFailed",
        message: "UTF-8 string allocation failed",
        fields: &[("RequestedBytes", Integer)],
    },
];
