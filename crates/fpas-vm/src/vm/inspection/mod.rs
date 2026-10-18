//! Read-only inspection of worker storage and portable executable metadata.

/// Initialization-aware binding and reference reads.
pub(super) mod bindings;
/// Bounded traversal of frame owners and call sites.
pub(super) mod frames;
/// Nonblocking local/shared global access.
pub(super) mod globals;
/// Portable source and lexical metadata.
pub(super) mod location;
/// Lazy value views and owned snapshots.
pub(super) mod values;

#[cfg(test)]
mod tests;

pub(in crate::vm) mod detached_frame;
