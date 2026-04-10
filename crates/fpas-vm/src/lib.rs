#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "VM tests use expect to keep low-level bytecode assertions focused on behavior"
    )
)]

mod vm;

pub use vm::{Vm, VmError, VmOutput};

#[cfg(test)]
mod tests;
