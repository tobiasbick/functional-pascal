use super::Value;

#[derive(Clone, Copy)]
enum RealComparison {
    Runtime,
    Constant,
}

impl RealComparison {
    fn equal(self, left: f64, right: f64) -> bool {
        match self {
            Self::Runtime if !left.is_nan() && !right.is_nan() => left == right,
            Self::Runtime | Self::Constant => left.to_bits() == right.to_bits(),
        }
    }
}

/// Structural equality for runtime values.
pub(super) fn values_equal(a: &Value, b: &Value) -> bool {
    compare_values(a, b, RealComparison::Runtime)
}

/// Structural identity for constant-pool entries.
pub(crate) fn constant_values_equal(a: &Value, b: &Value) -> bool {
    compare_values(a, b, RealComparison::Constant)
}

fn compare_values(a: &Value, b: &Value, real_comparison: RealComparison) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Real(x), Value::Real(y)) => real_comparison.equal(*x, *y),
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Enum(a), Value::Enum(b)) => {
            a.type_name == b.type_name
                && a.variant == b.variant
                && a.fields.len() == b.fields.len()
                && a.fields
                    .iter()
                    .zip(&b.fields)
                    .all(|(left, right)| compare_values(left, right, real_comparison))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(left, right)| compare_values(left, right, real_comparison))
        }
        (Value::Dict(a), Value::Dict(b)) => {
            a.len() == b.len()
                && a.iter().zip(b.iter()).all(|((ak, av), (bk, bv))| {
                    compare_values(ak, bk, real_comparison)
                        && compare_values(av, bv, real_comparison)
                })
        }
        (Value::Record(a), Value::Record(b)) => {
            a.type_name == b.type_name
                && a.fields.len() == b.fields.len()
                && a.fields
                    .iter()
                    .zip(&b.fields)
                    .all(|((an, av), (bn, bv))| an == bn && compare_values(av, bv, real_comparison))
        }
        (Value::Unit, Value::Unit) => true,
        (Value::ResultOk(a), Value::ResultOk(b)) => compare_values(a, b, real_comparison),
        (Value::ResultError(a), Value::ResultError(b)) => compare_values(a, b, real_comparison),
        (Value::OptionSome(a), Value::OptionSome(b)) => compare_values(a, b, real_comparison),
        (Value::OptionNone, Value::OptionNone) => true,
        (Value::Function(a), Value::Function(b)) => {
            a.name == b.name
                && a.task_bound == b.task_bound
                && a.captures.len() == b.captures.len()
                && a.captures
                    .iter()
                    .zip(&b.captures)
                    .all(|(left, right)| compare_values(left, right, real_comparison))
        }
        (Value::Cell(a), Value::Cell(b)) => std::sync::Arc::ptr_eq(a, b),
        (Value::Task(a), Value::Task(b)) => a == b,
        _ => false,
    }
}
