use super::super::{Fault, Lifecycle};
use crate::{
    RawJson,
    protocol::{v1, v2},
    schema::Version,
};

/// Independently enabled experimental protocol surfaces. Compilation grants none.
#[derive(Clone, Copy, Default)]
pub struct Experimental {
    /// Exact prompt-end token counters.
    pub token_usage: bool,
    /// Provider configuration methods.
    pub providers: bool,
    /// Explicit MCP endpoints and carrier messages.
    pub mcp: bool,
    /// ID-addressed experimental plan operations.
    pub plans: bool,
    /// Session forks.
    pub fork: bool,
    /// Client system prompt fields.
    pub system_prompt: bool,
    /// Restricted observed child sessions and inter-session messages.
    pub subagents: bool,
}
/// The initialization offer configured before a transport is started.
#[derive(Clone)]
pub struct Settings {
    /// Versions accepted by this client, without duplicates.
    pub supported_versions: Vec<Version>,
    /// Identity that the explicit initialize request must offer.
    pub client_info: Option<v1::Implementation>,
    /// Host capabilities offered on v1 connections.
    pub client_capabilities: v1::ClientCapabilities,
    /// Separately typed capabilities offered on v2 connections.
    pub v2_client_capabilities: v2::ClientCapabilities,
    /// Independent local experimental opt-ins.
    pub experimental: Experimental,
}
impl Settings {
    /// Offer one protocol family and no optional host capabilities.
    pub fn for_version(version: Version) -> Self {
        Self {
            supported_versions: vec![version],
            client_info: None,
            client_capabilities: Default::default(),
            v2_client_capabilities: Default::default(),
            experimental: Default::default(),
        }
    }
    /// Validate the offer, advertisement gates and supported set; select its highest version.
    pub fn requested_version(&self) -> Result<Version, Fault> {
        if self.supported_versions.is_empty()
            || self.supported_versions.len() > 2
            || (self.supported_versions.len() == 2
                && self.supported_versions[0] == self.supported_versions[1])
        {
            return Err(Fault::InvalidArgument("SupportedVersions"));
        }
        if self.client_capabilities.plan.is_some() && !self.experimental.plans {
            return Err(Fault::UnsupportedCapability("PlanOperations"));
        }
        if self.client_capabilities.subagents.is_some() && !self.experimental.subagents {
            return Err(Fault::UnsupportedCapability("Subagents"));
        }
        Ok(if self.supported_versions.contains(&Version::V2) {
            Version::V2
        } else {
            Version::V1
        })
    }
}
/// Detached negotiated state; original initialization data retains open metadata.
pub struct Snapshot {
    /// Lifecycle observed under the same lock as negotiation.
    pub lifecycle: Lifecycle,
    /// Highest configured protocol version.
    pub requested_version: Version,
    /// Actual selected protocol family, absent before initialization succeeds.
    pub negotiated_version: Option<Version>,
    /// Successful initialization result.
    pub initialization: Option<RawJson>,
}
