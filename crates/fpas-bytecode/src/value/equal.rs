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
        (Value::Enum(a), Value::Enum(b)) => {
            a.type_name == b.type_name
                && a.variant == b.variant
                && a.fields.len() == b.fields.len()
                && a.fields
                    .iter()
                    .zip(&b.fields)
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
                    .zip(b.iter())
                    .all(|((ak, av), (bk, bv))| values_equal(ak, bk) && values_equal(av, bv))
        }
        (Value::Record(a), Value::Record(b)) => {
            a.type_name == b.type_name
                && a.fields.len() == b.fields.len()
                && a.fields
                    .iter()
                    .zip(&b.fields)
                    .all(|((an, av), (bn, bv))| an == bn && values_equal(av, bv))
        }
        (Value::Unit, Value::Unit) => true,
        (Value::ResultOk(a), Value::ResultOk(b)) => values_equal(a, b),
        (Value::ResultError(a), Value::ResultError(b)) => values_equal(a, b),
        (Value::OptionSome(a), Value::OptionSome(b)) => values_equal(a, b),
        (Value::OptionNone, Value::OptionNone) => true,
        (Value::Function(a), Value::Function(b)) => {
            a.name == b.name
                && a.task_bound == b.task_bound
                && a.captures.len() == b.captures.len()
                && a.captures
                    .iter()
                    .zip(&b.captures)
                    .all(|(left, right)| values_equal(left, right))
        }
        (Value::Cell(a), Value::Cell(b)) => std::sync::Arc::ptr_eq(a, b),
        (Value::Task(a), Value::Task(b)) => a == b,
        _ => false,
    }
}
