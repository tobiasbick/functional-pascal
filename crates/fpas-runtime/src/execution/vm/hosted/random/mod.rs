//! VM-local `Std.Math.Random` intrinsic execution.
//!
//! **Documentation:** `docs/pascal/std/numeric/random.md` (from the repository root).

pub(super) mod state;

use crate::{Intrinsic, RandomIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_RANDOM_SOURCE_FAILURE, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::{VmError, worker::Worker};

#[derive(Clone, Copy)]
pub(super) enum Call {
    Real,
    Integer(i64, i64),
    Randomize,
    SetSeed(i64),
}

impl Call {
    pub(super) fn prepare(
        worker: &Worker,
        operation: RandomIntrinsic,
        arguments: &[Value],
    ) -> Result<Self, VmError> {
        match operation {
            RandomIntrinsic::Random => {
                argument_count(arguments, 0, worker)?;
                Ok(Self::Real)
            }
            RandomIntrinsic::RandomInt => {
                argument_count(arguments, 2, worker)?;
                let (lo, hi) = (
                    integer(arguments, 0, worker)?,
                    integer(arguments, 1, worker)?,
                );
                if lo > hi {
                    return Err(worker.runtime_error(
                        RUNTIME_NUMERIC_DOMAIN_ERROR,
                        format!("RandomInt lower bound {lo} must be <= upper bound {hi}"),
                        "Pass bounds where `Lo <= Hi` to Std.Math.Random.RandomInt.",
                    ));
                }
                Ok(Self::Integer(lo, hi))
            }
            RandomIntrinsic::Randomize => {
                argument_count(arguments, 0, worker)?;
                Ok(Self::Randomize)
            }
            RandomIntrinsic::SetSeed => {
                argument_count(arguments, 1, worker)?;
                Ok(Self::SetSeed(integer(arguments, 0, worker)?))
            }
        }
    }

    /// This path never contacts the operating system or waits for native work.
    pub(super) fn ready(self, state: &mut state::RandomState) -> Option<Value> {
        match self {
            Self::SetSeed(seed) => {
                state.set_seed(seed);
                Some(Value::Unit)
            }
            Self::Real if state.initialized() => Some(Value::Real(state.real())),
            Self::Integer(lo, hi) if state.initialized() => {
                Some(Value::Integer(state.integer(lo, hi)))
            }
            _ => None,
        }
    }

    pub(super) fn finish(
        self,
        state: &mut state::RandomState,
        seed: Result<[u8; 32], getrandom::Error>,
    ) -> Result<Value, getrandom::Error> {
        // A concurrent SetSeed/Randomize wins over an unfinished automatic seed.
        if !matches!(self, Self::Randomize)
            && let Some(value) = self.ready(state)
        {
            return Ok(value);
        }
        state.reseed(seed?);
        Ok(match self {
            Self::Real => Value::Real(state.real()),
            Self::Integer(lo, hi) => Value::Integer(state.integer(lo, hi)),
            Self::SetSeed(seed) => {
                state.set_seed(seed);
                Value::Unit
            }
            Self::Randomize => Value::Unit,
        })
    }
}

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
        let call = Call::prepare(self, operation, arguments)?;
        {
            let mut state = self.hosted.random.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(value) = call.ready(&mut state) {
                return Ok(Some(Some(value)));
            }
        }
        let mut seed = [0; 32];
        let seed = self.hosted.entropy_source.fill(&mut seed).map(|()| seed);
        let mut state = self.hosted.random.lock().unwrap_or_else(|p| p.into_inner());
        let value = call
            .finish(&mut state, seed)
            .map_err(|e| self.random_source_error(e))?;
        Ok(Some(Some(value)))
    }

    pub(super) fn random_source_error(&self, error: getrandom::Error) -> VmError {
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
                "Std.Math.Random intrinsic expected {expected} arguments, got {}",
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
            format!(
                "Std.Math.Random expected integer, got {}",
                actual.type_name()
            ),
            "Pass integer arguments to Std.Math.Random.RandomInt and Std.Math.Random.SetSeed.",
        )),
        None => Err(worker.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Std.Math.Random intrinsic argument is missing",
            "Check the compiler intrinsic signature and register argument count.",
        )),
    }
}
