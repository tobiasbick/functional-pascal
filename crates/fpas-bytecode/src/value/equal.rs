use super::Value;

/// Structural equality for runtime values.
pub(super) fn values_equal(a: &Value, b: &Value) -> bool {
    compare_values(a, b)
}

fn compare_values(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Real(x), Value::Real(y)) if !x.is_nan() && !y.is_nan() => x == y,
        (Value::Real(x), Value::Real(y)) => x.to_bits() == y.to_bits(),
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Enum(a), Value::Enum(b)) => {
            let a = a.body();
            let b = b.body();
            a.layout == b.layout
                && a.values.len() == b.values.len()
                && a.values
                    .iter()
                    .zip(&b.values)
                    .all(|(left, right)| compare_values(left, right))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(left, right)| compare_values(left, right))
        }
        (Value::Dict(a), Value::Dict(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|((ak, av), (bk, bv))| compare_values(ak, bk) && compare_values(av, bv))
        }
        (Value::Record(a), Value::Record(b)) => {
            let a = a.body();
            let b = b.body();
            a.layout == b.layout
                && a.values.len() == b.values.len()
                && a.values
                    .iter()
                    .zip(&b.values)
                    .all(|(left, right)| compare_values(left, right))
        }
        (Value::Unit, Value::Unit) => true,
        (Value::ResultOk(a), Value::ResultOk(b)) => compare_values(a, b),
        (Value::ResultError(a), Value::ResultError(b)) => compare_values(a, b),
        (Value::OptionSome(a), Value::OptionSome(b)) => compare_values(a, b),
        (Value::OptionNone, Value::OptionNone) => true,
        (Value::Function(a), Value::Function(b)) => {
            a.function == b.function
                && a.name == b.name
                && a.task_bound == b.task_bound
                && a.owner_task == b.owner_task
                && match (&a.bound_receiver, &b.bound_receiver) {
                    (Some(left), Some(right)) => compare_values(left, right),
                    (None, None) => true,
                    _ => false,
                }
                && a.captures.len() == b.captures.len()
                && a.captures
                    .iter()
                    .zip(&b.captures)
                    .all(|(left, right)| compare_values(left, right))
        }
        (Value::Cell(a), Value::Cell(b)) => std::sync::Arc::ptr_eq(a, b),
        (Value::Task(a), Value::Task(b)) => a == b,
        (Value::OpaqueHandle(a), Value::OpaqueHandle(b)) => a == b,
        _ => false,
    }
}
