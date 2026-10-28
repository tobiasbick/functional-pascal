//! VM-owned cooperative-cancellation state.

mod registry;

pub(in crate::execution::vm) use registry::{
    CancellationRegistry, CancellationView, OwnedCancellation,
};
