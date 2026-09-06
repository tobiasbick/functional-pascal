//! Per-VM bounded channel state.

mod registry;

pub(in crate::vm) use registry::{
    CANCELLATION_POLL_INTERVAL, ChannelRegistry, ReceiveState, SendState,
};
