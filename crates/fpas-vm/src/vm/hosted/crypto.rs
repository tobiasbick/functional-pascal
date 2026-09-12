//! Operating-system-backed `Std.Crypto` intrinsic execution.
//!
//! **Documentation:** `docs/pascal/std/cryptography/crypto.md` (from the repository root).

use fpas_bytecode::{CryptoIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::{VmError, worker::Worker};
use super::random::state::uniform_i64_with;

const MAX_RANDOM_BYTES: i64 = 1_048_576;

impl Worker {
    pub(super) fn execute_crypto_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        _location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Crypto(operation) = intrinsic else {
            return Ok(None);
        };
        let result = match operation {
            CryptoIntrinsic::RandomBytes => {
                argument_count(arguments, 1, self)?;
                random_bytes(integer(arguments, 0, self)?)
            }
            CryptoIntrinsic::RandomInt => {
                argument_count(arguments, 2, self)?;
                random_integer(integer(arguments, 0, self)?, integer(arguments, 1, self)?)
            }
        };
        Ok(Some(Some(match result {
            Ok(value) => Value::result_ok(value),
            Err(message) => Value::result_error(Value::Str(message.into())),
        })))
    }
}

fn random_bytes(count: i64) -> Result<Value, String> {
    if !(0..=MAX_RANDOM_BYTES).contains(&count) {
        return Err(format!(
            "Count must be in 0..={MAX_RANDOM_BYTES}, got {count}"
        ));
    }
    let mut bytes = vec![0_u8; count as usize];
    getrandom::fill(&mut bytes)
        .map_err(|error| format!("Operating-system random source failed: {error}"))?;
    Ok(Value::Array(
        bytes
            .into_iter()
            .map(|byte| Value::Integer(i64::from(byte)))
            .collect::<Vec<_>>()
            .into(),
    ))
}

fn random_integer(lo: i64, hi: i64) -> Result<Value, String> {
    if lo > hi {
        return Err(format!(
            "RandomInt lower bound {lo} must be <= upper bound {hi}"
        ));
    }
    uniform_i64_with(lo, hi, || getrandom::u64())
        .map(Value::Integer)
        .map_err(|error| format!("Operating-system random source failed: {error}"))
}

fn argument_count(arguments: &[Value], expected: usize, worker: &Worker) -> Result<(), VmError> {
    if arguments.len() == expected {
        Ok(())
    } else {
        Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Crypto intrinsic expected {expected} arguments, got {}",
                arguments.len()
            ),
            "Check the compiler intrinsic signature and register argument count.",
        ))
    }
}

fn integer(arguments: &[Value], index: usize, worker: &Worker) -> Result<i64, VmError> {
    match arguments.get(index) {
        Some(Value::Integer(value)) => Ok(*value),
        Some(actual) => Err(worker.runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("Std.Crypto expected integer, got {}", actual.type_name()),
            "Pass integer arguments to Std.Crypto random functions.",
        )),
        None => Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Std.Crypto intrinsic argument is missing",
            "Check the compiler intrinsic signature and register argument count.",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_bytes_enforces_the_public_allocation_limit() {
        assert!(random_bytes(-1).is_err());
        assert!(random_bytes(MAX_RANDOM_BYTES + 1).is_err());
        assert_eq!(random_bytes(0), Ok(Value::Array(Vec::new().into())));
    }

    #[test]
    fn random_integer_rejects_inverted_bounds() {
        assert!(random_integer(2, 1).is_err());
    }
}
