//! Shared field shapes for detached native records, including error payloads.

use crate::native_errors::ErrorFieldType;

/// A native record declaration shared by semantic registration and image admission.
pub struct NativeRecord {
    /// Fully qualified nominal identity.
    pub name: &'static str,
    /// Fields in declaration order.
    pub fields: &'static [(&'static str, ErrorFieldType)],
}

/// Detached records available to native value constructors.
pub const ALL: &[NativeRecord] = &[
    NativeRecord {
        name: "Std.Ai.Acp.Types.RawJson",
        fields: &[("Text", ErrorFieldType::String)],
    },
    NativeRecord {
        name: "Std.Ai.Acp.Types.UInt64",
        fields: &[("Digits", ErrorFieldType::String)],
    },
    NativeRecord {
        name: "Std.Ai.Acp.Types.ProtocolError",
        fields: &[
            ("Code", ErrorFieldType::Integer),
            ("Message", ErrorFieldType::String),
            (
                "Data",
                ErrorFieldType::OptionalRecord("Std.Ai.Acp.Types.RawJson"),
            ),
        ],
    },
];

/// Look up a registered nominal record without accepting a structural substitute.
#[must_use]
pub fn find(name: &str) -> Option<&'static NativeRecord> {
    ALL.iter()
        .find(|record| record.name.eq_ignore_ascii_case(name))
}
