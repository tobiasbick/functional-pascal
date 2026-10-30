//! Operating-system-backed `Std.Crypto` intrinsic execution.
//!
//! **Documentation:** `docs/pascal/std/cryptography/crypto.md` (from the repository root).

use crate::{CryptoIntrinsic, Intrinsic, SourceLocation, Value};
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
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Crypto(operation) = intrinsic else {
            return Ok(None);
        };
        let result = Request::prepare(self, operation, arguments)?.and_then(|request| {
            request
                .run(|bytes| self.hosted.entropy_source.fill(bytes))
                .map_err(CryptoFailure::Entropy)
        });
        Ok(Some(Some(self.crypto_result(operation, result, location)?)))
    }

    pub(super) fn crypto_result(
        &self,
        operation: CryptoIntrinsic,
        result: Result<Output, CryptoFailure>,
        location: SourceLocation,
    ) -> Result<Value, VmError> {
        Ok(match result {
            Ok(Output::Integer(value)) => Value::result_ok(Value::Integer(value)),
            Ok(Output::Bytes(bytes)) => Value::result_ok(Value::Array(
                bytes
                    .into_iter()
                    .map(|v| Value::Integer(i64::from(v)))
                    .collect::<Vec<_>>()
                    .into(),
            )),
            Err(error) => {
                use crate::stdlib::native_errors::{self, NativeErrorFamily};
                let (variant, fields) = match error {
                    CryptoFailure::Argument(name, reason) => {
                        (0, vec![Value::Str(name.into()), Value::Str(reason.into())])
                    }
                    CryptoFailure::Entropy(error) => {
                        let operation = match operation {
                            CryptoIntrinsic::RandomBytes => "Std.Crypto.RandomBytes",
                            CryptoIntrinsic::RandomInt => "Std.Crypto.RandomInt",
                        };
                        let cause = native_errors::host(
                            self.layouts.as_ref(),
                            operation,
                            &[],
                            error.raw_os_error(),
                            error.to_string(),
                            location,
                        )?;
                        (1, vec![cause])
                    }
                };
                Value::result_error(native_errors::value(
                    self.layouts.as_ref(),
                    NativeErrorFamily::Crypto,
                    variant,
                    fields,
                    location,
                )?)
            }
        })
    }
}

#[derive(Clone, Copy)]
pub(super) enum Request {
    Bytes(usize),
    Integer(i64, i64),
}
pub(super) enum Output {
    Bytes(Vec<u8>),
    Integer(i64),
}
impl Request {
    pub(super) fn prepare(
        worker: &Worker,
        operation: CryptoIntrinsic,
        arguments: &[Value],
    ) -> Result<Result<Self, CryptoFailure>, VmError> {
        Ok(match operation {
            CryptoIntrinsic::RandomBytes => {
                argument_count(arguments, 1, worker)?;
                let count = integer(arguments, 0, worker)?;
                validate_count(count).map(|()| Self::Bytes(count as usize))
            }
            CryptoIntrinsic::RandomInt => {
                argument_count(arguments, 2, worker)?;
                let (lo, hi) = (
                    integer(arguments, 0, worker)?,
                    integer(arguments, 1, worker)?,
                );
                validate_bounds(lo, hi).map(|()| Self::Integer(lo, hi))
            }
        })
    }
    pub(super) fn run<E>(
        self,
        mut fill: impl FnMut(&mut [u8]) -> Result<(), E>,
    ) -> Result<Output, E> {
        match self {
            Self::Bytes(count) => {
                let mut bytes = vec![0; count];
                fill(&mut bytes)?;
                Ok(Output::Bytes(bytes))
            }
            Self::Integer(lo, hi) => uniform_i64_with(lo, hi, || {
                let mut bytes = [0; 8];
                fill(&mut bytes)?;
                Ok(u64::from_le_bytes(bytes))
            })
            .map(Output::Integer),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum CryptoFailure {
    Argument(&'static str, String),
    Entropy(getrandom::Error),
}

fn validate_count(count: i64) -> Result<(), CryptoFailure> {
    if !(0..=MAX_RANDOM_BYTES).contains(&count) {
        return Err(CryptoFailure::Argument(
            "Count",
            format!("Count must be in 0..={MAX_RANDOM_BYTES}, got {count}"),
        ));
    }
    Ok(())
}

fn validate_bounds(lo: i64, hi: i64) -> Result<(), CryptoFailure> {
    if lo > hi {
        return Err(CryptoFailure::Argument(
            "Lo",
            format!("RandomInt lower bound {lo} must be <= upper bound {hi}"),
        ));
    }
    Ok(())
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
        assert!(validate_count(-1).is_err());
        assert!(validate_count(MAX_RANDOM_BYTES + 1).is_err());
        assert!(
            matches!(Request::Bytes(0).run(getrandom::fill), Ok(Output::Bytes(v)) if v.is_empty())
        );
    }

    #[test]
    fn entropy_adapter_preserves_the_backend_error_without_fallback() {
        let error = getrandom::Error::UNSUPPORTED;
        assert!(matches!(Request::Bytes(8).run(|_| Err(error)), Err(e) if e == error));
        assert!(matches!(
            validate_count(-1),
            Err(CryptoFailure::Argument("Count", _))
        ));
    }

    #[test]
    fn random_integer_rejects_inverted_bounds() {
        assert!(validate_bounds(2, 1).is_err());
    }
}
