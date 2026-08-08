#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        clippy::panic,
        reason = "VM tests use unwrap/expect/panic to keep low-level bytecode assertions focused on behavior"
    )
)]

mod vm;

pub use fpas_std::ScreenSnapshot;
pub use vm::{CallbackSession, Execution, ShutdownHandle, Vm, VmError, VmOutput};
