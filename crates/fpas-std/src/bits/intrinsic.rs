//! Borrowed-argument adapters for the six `Std.Bits` operations.
//!
//! Documentation: `docs/pascal/std/numeric/bits.md`.

use fpas_bytecode::{BitsIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, pop_int, pop_value};

/// Executes a `Std.Bits` intrinsic using the same primitives as integer operators.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let Intrinsic::Bits(operation) = intrinsic else {
        return Ok(None);
    };
    let right = pop_int(pop_value(call, location)?, location)?;
    let value = if operation == BitsIntrinsic::BitNot {
        super::bit_not(right)
    } else {
        let left = pop_int(pop_value(call, location)?, location)?;
        match operation {
            BitsIntrinsic::BitAnd => super::bit_and(left, right),
            BitsIntrinsic::BitOr => super::bit_or(left, right),
            BitsIntrinsic::BitXor => super::bit_xor(left, right),
            BitsIntrinsic::ShiftLeft | BitsIntrinsic::ShiftRight => {
                let shifted = if operation == BitsIntrinsic::ShiftLeft {
                    super::shift_left(left, right)
                } else {
                    super::shift_right(left, right)
                };
                shifted.ok_or_else(|| {
                    std_runtime_error(
                        RUNTIME_NUMERIC_DOMAIN_ERROR,
                        format!("Shift amount {right} is out of range (0..63)"),
                        "Use a shift amount between 0 and 63 inclusive.",
                        location,
                    )
                })?
            }
            BitsIntrinsic::BitNot => unreachable!("unary bit operation handled above"),
        }
    };
    call.push(Value::Integer(value));
    Ok(Some(()))
}
