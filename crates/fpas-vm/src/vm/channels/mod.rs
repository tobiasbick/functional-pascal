//! Per-VM bounded channel state.

mod registry;

pub(in crate::vm) use registry::{ChannelRegistry, ReceiveState, SendState};
