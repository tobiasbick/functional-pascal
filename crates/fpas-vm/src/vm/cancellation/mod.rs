//! VM-owned cooperative-cancellation state.

mod registry;

pub(in crate::vm) use registry::CancellationRegistry;
