//! Native process failure schema, including bounded partial captures.

use super::{ErrorFieldType::*, ErrorVariant};

pub(super) const VARIANTS: &[ErrorVariant] = &[
    ErrorVariant {
        name: "StartFailed",
        message: "Process operation failed",
        fields: &[("Cause", Host)],
    },
    ErrorVariant {
        name: "WaitFailed",
        message: "Process operation failed",
        fields: &[("Cause", Host)],
    },
    ErrorVariant {
        name: "InvalidArgument",
        message: "Process operation failed",
        fields: &[("Name", String), ("Reason", String)],
    },
    ErrorVariant {
        name: "IoFailure",
        message: "Process operation failed",
        fields: &[("Cause", Host)],
    },
    ErrorVariant {
        name: "Timeout",
        message: "Process timed out",
        fields: &[
            ("Operation", String),
            ("Stdout", String),
            ("Stderr", String),
        ],
    },
    ErrorVariant {
        name: "Cancelled",
        message: "Process cancelled",
        fields: &[
            ("Operation", String),
            ("Stdout", String),
            ("Stderr", String),
        ],
    },
    ErrorVariant {
        name: "LimitExceeded",
        message: "Process resource limit exceeded",
        fields: &[
            ("Operation", String),
            ("Resource", String),
            ("Limit", Integer),
            ("Stdout", String),
            ("Stderr", String),
        ],
    },
    ErrorVariant {
        name: "ExecutionFailed",
        message: "Process execution failed",
        fields: &[("Cause", Host), ("Stdout", String), ("Stderr", String)],
    },
];
