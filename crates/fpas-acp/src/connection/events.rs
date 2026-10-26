//! Detached events and terminal state; no callback or VM cells escape here.

use super::Fault;
use crate::{RawJson, binding::Value, schema::Version, transport::TransportStatus};

/// Opaque identity of a locally admitted request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestToken(pub(crate) u64);
impl RequestToken {
    /// Native handle representation, checked again by its owning connection.
    pub fn id(self) -> u64 {
        self.0
    }
}

/// Opaque identity of a deferred peer request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Responder(pub(crate) u64);
impl Responder {
    /// Native handle representation, checked again by its owning connection.
    pub fn id(self) -> u64 {
        self.0
    }
}

/// Connection lifetime; protocol initialization owns the middle transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    /// Connected, without protocol negotiation.
    Uninitialized,
    /// An initialize request is in flight.
    Initializing,
    /// Protocol negotiation succeeded.
    Ready,
    /// New effects are rejected while resources are being closed.
    Closing,
    /// Clean EOF or explicit local close completed.
    Closed,
    /// A visible protocol, resource or host failure ended the connection.
    Failed,
}

/// Final, bounded state retained until the connection is released.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Closed {
    /// Cleanup failed; ownership remains with the host's teardown registry.
    pub cleanup_fault: Option<Fault>,
    /// Failure, or None for clean EOF/local close.
    pub fault: Option<Fault>,
    /// Physical transport status, including bounded subprocess stderr.
    pub transport: TransportStatus,
}

/// One application event, paired with its exact original RPC document when present.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// Detached typed or diagnostic payload.
    pub payload: Payload,
    /// Complete original message including ignored envelope fields.
    pub raw: Option<RawJson>,
}

/// Versioned protocol data and terminal outcomes.
#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    /// V2 initialization selected this protocol family without an implicit retry.
    Initialized(RequestToken, Version, crate::binding::Value),
    /// Typed reverse-direction MCP outcome, absent from the SDK agent response enum.
    McpResponse(RequestToken, Version, crate::binding::Value),
    /// Successful response to a local request.
    Response(RequestToken, Version, Value),
    /// Peer error or terminal failure for a local request.
    RequestFailed(RequestToken, Fault),
    /// Recognized agent notification.
    Notification(Version, Value),
    /// Peer request with a separately retained single-use response right.
    Request(Responder, Version, Value),
    /// Peer cancellation expired a response right.
    RequestCancelled(Responder),
    /// Unrecognized notification retained for explicit application diagnosis.
    UnknownNotification(String, RawJson),
    /// The final event, delivered exactly once after request settlement.
    Closed(Closed),
}
