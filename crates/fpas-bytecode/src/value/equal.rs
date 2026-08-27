use super::Value;

/// Structural equality for runtime values.
pub(super) fn values_equal(a: &Value, b: &Value) -> bool {
    let mut pending = vec![(a, b)];
    while let Some((a, b)) = pending.pop() {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) if x == y => {}
            (Value::Real(x), Value::Real(y))
                if (!x.is_nan() && !y.is_nan() && x == y) || x.to_bits() == y.to_bits() => {}
            (Value::Boolean(x), Value::Boolean(y)) if x == y => {}
            (Value::Str(x), Value::Str(y)) if x == y => {}
            (Value::Enum(a), Value::Enum(b)) => {
                let a = a.body();
                let b = b.body();
                if a.layout != b.layout || a.values.len() != b.values.len() {
                    return false;
                }
                push_pairs(&mut pending, &a.values, &b.values);
            }
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                push_pairs(&mut pending, a, b);
            }
            (Value::Dict(a), Value::Dict(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for ((left_key, left_value), (right_key, right_value)) in
                    a.iter().zip(b.iter()).rev()
                {
                    pending.push((left_value, right_value));
                    pending.push((left_key, right_key));
                }
            }
            (Value::Record(a), Value::Record(b)) => {
                let a = a.body();
                let b = b.body();
                if a.layout != b.layout || a.values.len() != b.values.len() {
                    return false;
                }
                push_pairs(&mut pending, &a.values, &b.values);
            }
            (Value::Unit, Value::Unit) | (Value::OptionNone, Value::OptionNone) => {}
            (Value::ResultOk(a), Value::ResultOk(b))
            | (Value::ResultError(a), Value::ResultError(b))
            | (Value::OptionSome(a), Value::OptionSome(b)) => pending.push((a, b)),
            (Value::Function(a), Value::Function(b)) => {
                if a.function != b.function
                    || a.name != b.name
                    || a.task_bound != b.task_bound
                    || a.owner_task != b.owner_task
                    || a.captures.len() != b.captures.len()
                {
                    return false;
                }
                push_pairs(&mut pending, &a.captures, &b.captures);
                match (&a.bound_receiver, &b.bound_receiver) {
                    (Some(left), Some(right)) => pending.push((left, right)),
                    (None, None) => {}
                    _ => return false,
                }
            }
            (Value::Cell(a), Value::Cell(b)) if std::sync::Arc::ptr_eq(a, b) => {}
            (Value::Task(a), Value::Task(b)) if a == b => {}
            (Value::OpaqueHandle(a), Value::OpaqueHandle(b)) if a == b => {}
            _ => return false,
        }
    }
    true
}

fn push_pairs<'a>(
    pending: &mut Vec<(&'a Value, &'a Value)>,
    left: &'a [Value],
    right: &'a [Value],
) {
    pending.extend(left.iter().zip(right).rev());
}
