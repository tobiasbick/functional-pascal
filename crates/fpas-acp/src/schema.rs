//! Authoritative schema graphs derived from the pinned SDK, including supporting values.

use std::sync::LazyLock;

use crate::protocol::{self, v1, v2};

/// Versioned protocol family; compiling v2 does not negotiate or advertise it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Version {
    /// Stable ACP v1 with separately guarded experimental fields represented.
    V1,
    /// Explicitly selected draft ACP v2.
    V2,
}

/// Complete SDK-derived graph reached from all client-role request/response/update roots.
#[must_use]
pub fn document(version: Version) -> &'static serde_json::Value {
    static V1: LazyLock<serde_json::Value> = LazyLock::new(|| {
        schemars::schema_for!((
            v1::ClientRequest,
            v1::AgentResponse,
            v1::ClientNotification,
            v1::AgentRequest,
            v1::ClientResponse,
            v1::AgentNotification,
            v1::RequestId,
            v1::Error,
            protocol::ProtocolVersion,
            v1::ProtocolLevelNotification,
        ))
        .to_value()
    });
    static V2: LazyLock<serde_json::Value> = LazyLock::new(|| {
        schemars::schema_for!((
            v2::ClientRequest,
            v2::AgentResponse,
            v2::ClientNotification,
            v2::AgentRequest,
            v2::ClientResponse,
            v2::AgentNotification,
            v2::RequestId,
            v2::Error,
            protocol::ProtocolVersion,
            v2::ProtocolLevelNotification,
        ))
        .to_value()
    });
    match version {
        Version::V1 => &V1,
        Version::V2 => &V2,
    }
}
