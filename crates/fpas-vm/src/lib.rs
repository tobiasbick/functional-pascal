#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::panic,
        reason = "VM tests use unwrap/expect/panic to keep low-level bytecode assertions focused on behavior"
    )
)]

mod vm;

pub use fpas_std::ScreenSnapshot;
pub use vm::{
    RegisterCallbackSession, RegisterExecution, RegisterShutdownHandle, RegisterVm, Vm, VmError,
    VmOutput, VmShutdownHandle,
};

#[cfg(test)]
mod tests;
