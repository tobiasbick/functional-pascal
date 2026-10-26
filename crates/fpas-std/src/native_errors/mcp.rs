//! MCP failures without protocol or credential payloads in ordinary display.
use super::{ErrorFieldType::*, ErrorVariant};
pub(super) const VARIANTS: &[ErrorVariant] = &[
    ErrorVariant {
        name: "InvalidArgument",
        message: "Invalid MCP argument",
        fields: &[("Argument", String)],
    },
    ErrorVariant {
        name: "InvalidSchema",
        message: "Invalid MCP tool schema",
        fields: &[("Schema", String)],
    },
    ErrorVariant {
        name: "DuplicateTool",
        message: "MCP tool name is already registered",
        fields: &[],
    },
    ErrorVariant {
        name: "UnknownTool",
        message: "Unknown MCP tool",
        fields: &[],
    },
    ErrorVariant {
        name: "InvalidArguments",
        message: "MCP arguments do not match the input schema",
        fields: &[],
    },
    ErrorVariant {
        name: "InvalidResult",
        message: "Invalid MCP tool result",
        fields: &[],
    },
    ErrorVariant {
        name: "LimitExceeded",
        message: "MCP resource limit exceeded",
        fields: &[("Resource", String), ("Limit", Integer)],
    },
    ErrorVariant {
        name: "Busy",
        message: "MCP server already has a serving task",
        fields: &[],
    },
    ErrorVariant {
        name: "Closed",
        message: "MCP resource is closed",
        fields: &[],
    },
    ErrorVariant {
        name: "Transport",
        message: "MCP transport failed",
        fields: &[("Kind", String)],
    },
    ErrorVariant {
        name: "Cancelled",
        message: "MCP serving was cancelled",
        fields: &[],
    },
    ErrorVariant {
        name: "Timeout",
        message: "MCP operation timed out",
        fields: &[],
    },
    ErrorVariant {
        name: "PermissionDenied",
        message: "MCP application access denied",
        fields: &[],
    },
];
