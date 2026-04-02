//! `Std.Math.*` intrinsic implementations (`Pi` is compile-time in `fpas-compiler`).
//!
//! **Documentation:** `docs/pascal/std/math.md` (from the repository root).
//! **Maintenance:** Keep that Markdown file aligned with this file, `intrinsics.rs`,
//! `fpas-bytecode::Intrinsic`, `fpas-compiler` (`Std.Math.Pi` and call lowering), and `fpas-sema` `std_registry.rs`.

use crate::error::{StdError, std_runtime_error};
use crate::helpers::{pop_int, pop_real, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_NUMERIC_DOMAIN_ERROR};
use rand::Rng;

const INTEGER_RANGE_HINT: &str = "Use a finite value whose rounded result fits in the integer range -9223372036854775808..9223372036854775807.";

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
            stack.push(Value::Integer(checked_real_to_int(
                r.floor(),
                "Floor",
                location,
            )?));
        }
        Intrinsic::MathCeil => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(checked_real_to_int(
                r.ceil(),
                "Ceil",
                location,
            )?));
        }
        Intrinsic::MathRound => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(checked_real_to_int(
                r.round(),
                "Round",
                location,
            )?));
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
        Intrinsic::MathTan => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(r.tan()));
        }
        Intrinsic::MathArcSin => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            if !(-1.0..=1.0).contains(&r) {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    format!("ArcSin argument {r} out of range [-1, 1]"),
                    "Ensure the argument to Std.Math.ArcSin is in [-1, 1].",
                    location,
                ));
            }
            stack.push(Value::Real(r.asin()));
        }
        Intrinsic::MathArcCos => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            if !(-1.0..=1.0).contains(&r) {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    format!("ArcCos argument {r} out of range [-1, 1]"),
                    "Ensure the argument to Std.Math.ArcCos is in [-1, 1].",
                    location,
                ));
            }
            stack.push(Value::Real(r.acos()));
        }
        Intrinsic::MathArcTan => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(r.atan()));
        }
        Intrinsic::MathArcTan2 => {
            let x = pop_real(pop_value(stack, location)?, location)?;
            let y = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(y.atan2(x)));
        }
        Intrinsic::MathExp => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(r.exp()));
        }
        Intrinsic::MathLog10 => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            if r <= 0.0 {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    "Log10 expects a positive real",
                    "Ensure the argument to Std.Math.Log10 is > 0.",
                    location,
                ));
            }
            stack.push(Value::Real(r.log10()));
        }
        Intrinsic::MathLog2 => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            if r <= 0.0 {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    "Log2 expects a positive real",
                    "Ensure the argument to Std.Math.Log2 is > 0.",
                    location,
                ));
            }
            stack.push(Value::Real(r.log2()));
        }
        Intrinsic::MathTrunc => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Integer(checked_real_to_int(
                r.trunc(),
                "Trunc",
                location,
            )?));
        }
        Intrinsic::MathFrac => {
            let r = pop_real(pop_value(stack, location)?, location)?;
            stack.push(Value::Real(r.fract()));
        }
        Intrinsic::MathSign => {
            let v = pop_value(stack, location)?;
            stack.push(sign_value(v, location)?);
        }
        Intrinsic::MathClamp => {
            let hi = pop_value(stack, location)?;
            let lo = pop_value(stack, location)?;
            let x = pop_value(stack, location)?;
            stack.push(clamp_value(x, lo, hi, location)?);
        }
        Intrinsic::MathRandom => {
            let mut rng = rand::thread_rng();
            let r: f64 = rng.r#gen();
            stack.push(Value::Real(r));
        }
        Intrinsic::MathRandomInt => {
            let hi = pop_int(pop_value(stack, location)?, location)?;
            let lo = pop_int(pop_value(stack, location)?, location)?;
            if lo > hi {
                return Err(std_runtime_error(
                    RUNTIME_NUMERIC_DOMAIN_ERROR,
                    format!("RandomInt lower bound {lo} must be <= upper bound {hi}"),
                    "Pass bounds where `Lo <= Hi` to Std.Math.RandomInt.",
                    location,
                ));
            }
            let mut rng = rand::thread_rng();
            let r: i64 = rng.gen_range(lo..=hi);
            stack.push(Value::Integer(r));
        }
        Intrinsic::MathRandomize => {
            // Randomize is a no-op when using thread_rng (automatically seeded).
            stack.push(Value::Unit);
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn abs_value(v: Value, location: SourceLocation) -> Result<Value, StdError> {
    match v {
        Value::Integer(n) => n.checked_abs().map(Value::Integer).ok_or_else(|| {
            std_runtime_error(
                RUNTIME_NUMERIC_DOMAIN_ERROR,
                "Abs overflow for minimum integer value",
                "Use a wider numeric type or avoid taking Abs of -9223372036854775808.",
                location,
            )
        }),
        Value::Real(x) => Ok(Value::Real(x.abs())),
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Abs expects numeric value, got {}", other.type_name()),
            "Pass an integer or real value to Std.Math.Abs.",
            location,
        )),
    }
}

fn checked_real_to_int(
    value: f64,
    intrinsic_name: &str,
    location: SourceLocation,
) -> Result<i64, StdError> {
    if !value.is_finite() || value < i64::MIN as f64 || value > i64::MAX as f64 {
        return Err(std_runtime_error(
            RUNTIME_NUMERIC_DOMAIN_ERROR,
            format!("{intrinsic_name} result {value} is outside the integer range"),
            INTEGER_RANGE_HINT,
            location,
        ));
    }

    Ok(value as i64)
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
        _ => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Min/Max expects two integers or two reals, got {} and {}",
                a.type_name(),
                b.type_name()
            ),
            "Both arguments must be the same numeric kind.",
            location,
        )),
    }
}

fn sign_value(v: Value, location: SourceLocation) -> Result<Value, StdError> {
    match v {
        Value::Integer(n) => Ok(Value::Integer(n.signum())),
        Value::Real(x) => {
            if x > 0.0 {
                Ok(Value::Integer(1))
            } else if x < 0.0 {
                Ok(Value::Integer(-1))
            } else {
                Ok(Value::Integer(0))
            }
        }
        other => Err(std_runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!("Sign expects numeric value, got {}", other.type_name()),
            "Pass an integer or real value to Std.Math.Sign.",
            location,
        )),
    }
}

fn clamp_value(
    x: Value,
    lo: Value,
    hi: Value,
    location: SourceLocation,
) -> Result<Value, StdError> {
    if let (Value::Integer(v), Value::Integer(a), Value::Integer(b)) = (&x, &lo, &hi) {
        ensure_valid_clamp_bounds(*a, *b, location)?;
        return Ok(Value::Integer((*v).clamp(*a, *b)));
    }
    if let (Value::Real(v), Value::Real(a), Value::Real(b)) = (&x, &lo, &hi) {
        ensure_valid_clamp_bounds(*a, *b, location)?;
        return Ok(Value::Real(v.clamp(*a, *b)));
    }
    Err(std_runtime_error(
        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
        "Clamp expects all three arguments to be the same numeric kind",
        "All arguments must be integer or all must be real.",
        location,
    ))
}

fn ensure_valid_clamp_bounds<T>(lo: T, hi: T, location: SourceLocation) -> Result<(), StdError>
where
    T: PartialOrd + std::fmt::Display,
{
    if lo <= hi {
        Ok(())
    } else {
        Err(std_runtime_error(
            RUNTIME_NUMERIC_DOMAIN_ERROR,
            format!("Clamp lower bound {lo} must be <= upper bound {hi}"),
            "Pass bounds where `Lo <= Hi` to Std.Math.Clamp(X, Lo, Hi).",
            location,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use fpas_bytecode::{Intrinsic, SourceLocation, Value};

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn abs_reports_overflow_for_min_integer() {
        let mut stack = vec![Value::Integer(i64::MIN)];

        let error = run(Intrinsic::MathAbs, &mut stack, test_location()).unwrap_err();

        assert!(error.message.contains("Abs overflow"), "{}", error.message);
    }

    #[test]
    fn floor_rejects_non_finite_values() {
        let mut stack = vec![Value::Real(f64::INFINITY)];

        let error = run(Intrinsic::MathFloor, &mut stack, test_location()).unwrap_err();

        assert!(error.message.contains("Floor result"), "{}", error.message);
    }

    #[test]
    fn trunc_rejects_out_of_range_values() {
        let mut stack = vec![Value::Real(1.0e300)];

        let error = run(Intrinsic::MathTrunc, &mut stack, test_location()).unwrap_err();

        assert!(error.message.contains("Trunc result"), "{}", error.message);
    }

    #[test]
    fn floor_ceil_and_trunc_keep_negative_finite_semantics() {
        let mut floor_stack = vec![Value::Real(-3.2)];
        run(Intrinsic::MathFloor, &mut floor_stack, test_location()).unwrap();
        assert_eq!(floor_stack, vec![Value::Integer(-4)]);

        let mut ceil_stack = vec![Value::Real(-3.2)];
        run(Intrinsic::MathCeil, &mut ceil_stack, test_location()).unwrap();
        assert_eq!(ceil_stack, vec![Value::Integer(-3)]);

        let mut trunc_stack = vec![Value::Real(-3.7)];
        run(Intrinsic::MathTrunc, &mut trunc_stack, test_location()).unwrap();
        assert_eq!(trunc_stack, vec![Value::Integer(-3)]);
    }

    #[test]
    fn round_accepts_regular_finite_values() {
        let mut stack = vec![Value::Real(2.6)];

        run(Intrinsic::MathRound, &mut stack, test_location()).unwrap();

        assert_eq!(stack, vec![Value::Integer(3)]);
    }
}
