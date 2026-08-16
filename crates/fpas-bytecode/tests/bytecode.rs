//! Register-bytecode representation and verifier integration tests.

#![expect(
    clippy::expect_used,
    reason = "test-only executable builders fail fast when a fixture is malformed"
)]

mod bytecode {
    mod effects;
    mod executable;
    mod instruction;
    mod instruction_change;
    mod support;
    mod verifier;
}
