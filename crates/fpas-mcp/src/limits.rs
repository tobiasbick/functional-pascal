//! Resource ceilings validated before catalog or dispatcher allocation.

use crate::Fault;

/// Per-server limits for catalogs and detached application calls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Registered tool definitions.
    pub tools: usize,
    /// Serialized array of registered tool definitions.
    pub catalog_bytes: usize,
    /// Complete serialized call parameters including metadata, or one result.
    pub message_bytes: usize,
    /// Calls retained from admission through terminal response.
    pub pending_calls: usize,
    /// Retained argument, request identity and metadata representations across all calls.
    pub pending_bytes: usize,
    /// Independent application or ACP attachments.
    pub endpoints: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            tools: 128,
            catalog_bytes: 1024 * 1024,
            message_bytes: 1024 * 1024,
            pending_calls: 64,
            pending_bytes: 8 * 1024 * 1024,
            endpoints: 16,
        }
    }
}

impl Limits {
    /// Reject impossible or excessive limits before accepting registrations.
    pub fn validate(&self) -> Result<(), Fault> {
        for (name, value, maximum) in [
            ("Tools", self.tools, 4096),
            ("CatalogBytes", self.catalog_bytes, 64 * 1024 * 1024),
            ("MessageBytes", self.message_bytes, 64 * 1024 * 1024),
            ("PendingCalls", self.pending_calls, 4096),
            ("PendingBytes", self.pending_bytes, 64 * 1024 * 1024),
            ("Endpoints", self.endpoints, 4096),
        ] {
            if value == 0 || value > maximum {
                return Err(Fault::InvalidArgument(name));
            }
        }
        Ok(())
    }
}
