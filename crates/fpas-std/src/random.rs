//! `Std.Random.*` intrinsic implementations.
//!
//! **Documentation:** `docs/pascal/std/numeric/random.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{pop_int, pop_value};
use fpas_bytecode::{Intrinsic, RandomIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;
use rand::Rng;

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Random(RandomIntrinsic::Random) => {
            let mut rng = rand::rng();
            let random_real: f64 = rng.random();
            stack.push(Value::Real(random_real));
        }
        Intrinsic::Random(RandomIntrinsic::RandomInt) => {
            let upper_bound = pop_int(pop_value(stack, location)?, location)?;
            let lower_bound = pop_int(pop_value(stack, location)?, location)?;
            if lower_bound > upper_bound {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    format!(
                        "RandomInt lower bound {lower_bound} must be <= upper bound {upper_bound}"
                    ),
                    "Pass bounds where `Lo <= Hi` to Std.Random.RandomInt.",
                    location,
                ));
            }
            let mut rng = rand::rng();
            let random_integer: i64 = rng.random_range(lower_bound..=upper_bound);
            stack.push(Value::Integer(random_integer));
        }
        Intrinsic::Random(RandomIntrinsic::Randomize) => {
            stack.push(Value::Unit);
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}
