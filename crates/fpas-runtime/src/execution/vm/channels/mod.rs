//! Per-VM bounded channel state.

mod registry;

pub(in crate::execution::vm) use registry::{
    CANCELLATION_POLL_INTERVAL, ChannelRegistry, ReceiveState, SendState,
};
