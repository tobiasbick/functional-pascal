use super::{ErrorFieldType::*, ErrorVariant};

pub(super) const VARIANTS: &[ErrorVariant] = &[
    ErrorVariant {
        name: "InvalidPattern",
        message: "Invalid regular expression",
        fields: &[("Detail", String)],
    },
    ErrorVariant {
        name: "InvalidStart",
        message: "Invalid regex start position",
        fields: &[("Start", Integer), ("Length", Integer)],
    },
    ErrorVariant {
        name: "InvalidLimit",
        message: "Invalid regex result limit",
        fields: &[("Limit", Integer)],
    },
    ErrorVariant {
        name: "LimitExceeded",
        message: "Regex resource limit exceeded",
        fields: &[("Name", String), ("Limit", Integer)],
    },
];
