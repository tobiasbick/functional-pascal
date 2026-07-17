use super::Value;

/// Compare two IEEE-754 reals for [`Value`] equality and constant-pool deduplication.
fn reals_equal(a: f64, b: f64) -> bool {
    if a.is_nan() || b.is_nan() {
        a.to_bits() == b.to_bits()
    } else {
        a == b
    }
}

/// Structural equality for runtime values.
pub(super) fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Real(x), Value::Real(y)) => reals_equal(*x, *y),
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (
            Value::Enum {
                type_name: a_type,
                variant: a_variant,
                fields: a_fields,
            },
            Value::Enum {
                type_name: b_type,
                variant: b_variant,
                fields: b_fields,
            },
        ) => {
            a_type == b_type
                && a_variant == b_variant
                && a_fields.len() == b_fields.len()
                && a_fields
                    .iter()
                    .zip(b_fields)
                    .all(|(left, right)| values_equal(left, right))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(left, right)| values_equal(left, right))
        }
        (Value::Dict(a), Value::Dict(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|((ak, av), (bk, bv))| values_equal(ak, bk) && values_equal(av, bv))
        }
        (
            Value::Record {
                type_name: a_type,
                fields: a_fields,
            },
            Value::Record {
                type_name: b_type,
                fields: b_fields,
            },
        ) => {
            a_type == b_type
                && a_fields.len() == b_fields.len()
                && a_fields
                    .iter()
                    .zip(b_fields)
                    .all(|((an, av), (bn, bv))| an == bn && values_equal(av, bv))
        }
        (Value::Unit, Value::Unit) => true,
        (Value::ResultOk(a), Value::ResultOk(b)) => values_equal(a, b),
        (Value::ResultError(a), Value::ResultError(b)) => values_equal(a, b),
        (Value::OptionSome(a), Value::OptionSome(b)) => values_equal(a, b),
        (Value::OptionNone, Value::OptionNone) => true,
        (
            Value::Function {
                name: a_name,
                captures: a_captures,
                task_bound: a_bound,
            },
            Value::Function {
                name: b_name,
                captures: b_captures,
                task_bound: b_bound,
            },
        ) => {
            a_name == b_name
                && a_bound == b_bound
                && a_captures.len() == b_captures.len()
                && a_captures
                    .iter()
                    .zip(b_captures)
                    .all(|(left, right)| values_equal(left, right))
        }
        (Value::Cell(a), Value::Cell(b)) => std::sync::Arc::ptr_eq(a, b),
        (Value::Task(a), Value::Task(b)) => a == b,
        _ => false,
    }
}
