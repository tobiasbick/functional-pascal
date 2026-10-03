//! Operations on signed integers viewed as 64-bit bit patterns.
//!
//! Documentation: `docs/pascal/std/numeric/bits.md`.
use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, pop_int, pop_value};
use fpas_bytecode::{BitsIntrinsic, Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;

/// Executes a bit operation, checking shift counts without numeric overflow checks.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let Intrinsic::Bits(op) = intrinsic else {
        return Ok(None);
    };
    let right = pop_int(pop_value(call, location)?, location)?;
    let result = if op == BitsIntrinsic::BitNot {
        !right
    } else {
        let left = pop_int(pop_value(call, location)?, location)?;
        match op {
            BitsIntrinsic::BitAnd => left & right,
            BitsIntrinsic::BitOr => left | right,
            BitsIntrinsic::BitXor => left ^ right,
            BitsIntrinsic::ShiftLeft | BitsIntrinsic::ShiftRight => {
                let count = u32::try_from(right)
                    .ok()
                    .filter(|n| *n < 64)
                    .ok_or_else(|| {
                        std_runtime_error(
                            RUNTIME_NUMERIC_DOMAIN_ERROR,
                            format!("Shift count {right} is out of range (0..63)"),
                            "Use an integer shift count between 0 and 63 inclusive.",
                            location,
                        )
                    })?;
                if op == BitsIntrinsic::ShiftLeft {
                    left.wrapping_shl(count)
                } else {
                    ((left as u64) >> count) as i64
                }
            }
            BitsIntrinsic::BitNot => unreachable!("unary operation handled above"),
        }
    };
    call.push(Value::Integer(result));
    Ok(Some(()))
}

#[cfg(test)]
mod tests;
