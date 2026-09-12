//! VM-local `Std.Random` intrinsic execution.
//!
//! **Documentation:** `docs/pascal/std/numeric/random.md` (from the repository root).

pub(super) mod state;

use fpas_bytecode::{Intrinsic, RandomIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_RANDOM_SOURCE_FAILURE, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::{VmError, worker::Worker};

impl Worker {
    pub(super) fn execute_random_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        _location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Random(operation) = intrinsic else {
            return Ok(None);
        };
        let mut state = self
            .hosted
            .random
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = match operation {
            RandomIntrinsic::Random => {
                argument_count(arguments, 0, self)?;
                Some(Value::Real(
                    state
                        .real()
                        .map_err(|error| self.random_source_error(error))?,
                ))
            }
            RandomIntrinsic::RandomInt => {
                argument_count(arguments, 2, self)?;
                let lo = integer(arguments, 0, self)?;
                let hi = integer(arguments, 1, self)?;
                if lo > hi {
                    return Err(self.runtime_error(
                        RUNTIME_NUMERIC_DOMAIN_ERROR,
                        format!("RandomInt lower bound {lo} must be <= upper bound {hi}"),
                        "Pass bounds where `Lo <= Hi` to Std.Random.RandomInt.",
                    ));
                }
                Some(Value::Integer(
                    state
                        .integer(lo, hi)
                        .map_err(|error| self.random_source_error(error))?,
                ))
            }
            RandomIntrinsic::Randomize => {
                argument_count(arguments, 0, self)?;
                state
                    .randomize()
                    .map_err(|error| self.random_source_error(error))?;
                Some(Value::Unit)
            }
            RandomIntrinsic::SetSeed => {
                argument_count(arguments, 1, self)?;
                state.set_seed(integer(arguments, 0, self)?);
                Some(Value::Unit)
            }
        };
        Ok(Some(value))
    }

    fn random_source_error(&self, error: getrandom::Error) -> VmError {
        self.runtime_error(
            RUNTIME_RANDOM_SOURCE_FAILURE,
            format!("Operating-system random source failed: {error}"),
            "Retry the operation after checking operating-system random-source availability.",
        )
    }
}

fn argument_count(arguments: &[Value], expected: usize, worker: &Worker) -> Result<(), VmError> {
    if arguments.len() == expected {
        Ok(())
    } else {
        Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Random intrinsic expected {expected} arguments, got {}",
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
            format!("Std.Random expected integer, got {}", actual.type_name()),
            "Pass integer arguments to Std.Random.RandomInt and Std.Random.SetSeed.",
        )),
        None => Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Std.Random intrinsic argument is missing",
            "Check the compiler intrinsic signature and register argument count.",
        )),
    }
}
