#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "Protocol fixtures fail directly at their observable boundary"
)]
//! Stable client contracts exercised through the production SDK message endpoint.
#[path = "v1/cancellation.rs"]
mod cancellation;
#[path = "v1/client.rs"]
mod client;
#[path = "v1/content.rs"]
mod content;
#[path = "v1/peer.rs"]
mod peer;
