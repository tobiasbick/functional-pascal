//! ACP operational failures, independent of protocol payload text.

use super::{ErrorFieldType::*, ErrorVariant};

pub(super) const VARIANTS: &[ErrorVariant] = &[
    ErrorVariant {
        name: "InvalidArgument",
        message: "Invalid ACP argument",
        fields: &[("Name", String), ("Reason", String)],
    },
    ErrorVariant {
        name: "UnsupportedCapability",
        message: "ACP capability is unavailable",
        fields: &[("Name", String)],
    },
    ErrorVariant {
        name: "UnsupportedVersion",
        message: "Unsupported ACP version",
        fields: &[("Version", Integer)],
    },
    ErrorVariant {
        name: "Busy",
        message: "ACP operation is busy",
        fields: &[("Operation", String)],
    },
    ErrorVariant {
        name: "Closed",
        message: "ACP resource is closed",
        fields: &[("Operation", String)],
    },
    ErrorVariant {
        name: "Timeout",
        message: "ACP operation timed out",
        fields: &[("Operation", String)],
    },
    ErrorVariant {
        name: "Cancelled",
        message: "ACP operation was cancelled",
        fields: &[("Operation", String)],
    },
    ErrorVariant {
        name: "LimitExceeded",
        message: "ACP limit exceeded",
        fields: &[("Name", String), ("Limit", Integer)],
    },
    ErrorVariant {
        name: "InvalidMessage",
        message: "Invalid ACP message",
        fields: &[
            ("Line", OptionalInteger),
            ("Column", OptionalInteger),
            ("Reason", String),
        ],
    },
    ErrorVariant {
        name: "Host",
        message: "ACP host operation failed",
        fields: &[("Cause", Host)],
    },
    ErrorVariant {
        name: "Peer",
        message: "ACP peer returned an error",
        fields: &[("Protocol", Record("Std.Ai.Acp.Types.ProtocolError"))],
    },
];
