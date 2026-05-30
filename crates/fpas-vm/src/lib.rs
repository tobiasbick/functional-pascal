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

pub use vm::{Vm, VmError, VmOutput};

#[cfg(test)]
mod tests;
