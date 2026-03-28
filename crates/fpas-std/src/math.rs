//! `Std.Math.*` intrinsic implementations (`Pi` is compile-time in `fpas-compiler`).
//!
//! **Documentation:** `docs/pascal/std/math.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler` (`Std.Math.Pi` and call lowering), and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{pop_real, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR};

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::MathSqrt => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            if r < 0.0 {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    "Sqrt of negative number",
                    "Ensure the argument to Std.Math.Sqrt is >= 0.",
                    location,
                ));
            }
            stack.push(Value::Real(r.sqrt()));
        }
        Intrinsic::MathPow => {
            let exp = pop_real(pop_value(stack, location)?, location)?;
            let base = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(base.powf(exp)));
        }
        Intrinsic::MathFloor => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(r.floor() as i64));
        }
        Intrinsic::MathCeil => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(r.ceil() as i64));
        }
        Intrinsic::MathRound => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(r.round() as i64));
        }
        Intrinsic::MathSin => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(r.sin()));
        }
        Intrinsic::MathCos => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(r.cos()));
        }
        Intrinsic::MathLog => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            if r <= 0.0 {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    "Log expects a positive real",
                    "Ensure the argument to Std.Math.Log is > 0.",
                    location,
                ));
            }
            stack.push(Value::Real(r.ln()));
        }
        Intrinsic::MathMin => {
            let b = pop_value(stack, location)?;
            let a = pop_value(stack, location)?;
            stack.push(minmax_value(a, b, true, location)?);
        }
        Intrinsic::MathMax => {
            let b = pop_value(stack, location)?;
            let a = pop_value(stack, location)?;
            stack.push(minmax_value(a, b, false, location)?);
        }
        Intrinsic::MathAbs => {
            let v = pop_value(stack, location)?;
            stack.push(abs_value(v, location)?);
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn abs_value(v: Value, location: SourceLocation) -> Result<Value, StdError> {
    match v {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Real(x) => Ok(Value::Real(x.abs())),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Abs expects numeric value, got {}", other.type_name()),
            "Pass an integer or real value to Std.Math.Abs.",
            location,
        )),
    }
}

fn minmax_value(
    a: Value,
    b: Value,
    min: bool,
    location: SourceLocation,
) -> Result<Value, StdError> {
    match (&a, &b) {
        (Value::Integer(x), Value::Integer(y)) => Ok(if min {
            Value::Integer(*x.min(y))
        } else {
            Value::Integer(*x.max(y))
        }),
        (Value::Real(x), Value::Real(y)) => Ok(if min {
            Value::Real(x.min(*y))
        } else {
            Value::Real(x.max(*y))
        }),
        (Value::Integer(x), Value::Real(y)) => {
            let xr = *x as f64;
            Ok(if min {
                Value::Real(xr.min(*y))
            } else {
                Value::Real(xr.max(*y))
            })
        }
        (Value::Real(x), Value::Integer(y)) => {
            let yr = *y as f64;
            Ok(if min {
                Value::Real(x.min(yr))
            } else {
                Value::Real(x.max(yr))
            })
        }
        _ => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Min/Max expects two integers or two reals (or mixed int/real), got {} and {}",
                a.type_name(),
                b.type_name()
            ),
            "Pass numeric values to Std.Math.Min/Std.Math.Max.",
            location,
        )),
    }
}
