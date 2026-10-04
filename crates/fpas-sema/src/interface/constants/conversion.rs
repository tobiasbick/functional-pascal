//! Scalar values in compiled-unit interfaces.

use fpas_bytecode::Value;
use fpas_ir::Constant;
use fpas_unit::interface::ConstantValue;

/// Persist a scalar value without changing its floating-point bits.
pub(crate) fn to_interface(value: Constant) -> Option<ConstantValue> {
    Some(match value {
        Constant::Integer(value) => ConstantValue::Integer(value),
        Constant::Real(value) => ConstantValue::Real(value.to_bits()),
        Constant::Boolean(value) => ConstantValue::Boolean(value),
        Constant::String(value) => ConstantValue::String(value),
        Constant::Unit => return None,
    })
}

/// Restore a scalar constant from a compiled-unit interface.
pub(super) fn from_interface(value: &ConstantValue) -> Option<Constant> {
    Some(match value {
        ConstantValue::Integer(value) => Constant::Integer(*value),
        ConstantValue::Real(bits) => Constant::Real(f64::from_bits(*bits)),
        ConstantValue::Boolean(value) => Constant::Boolean(*value),
        ConstantValue::String(value) => Constant::String(value.clone()),
        ConstantValue::EnumValue { backing_value, .. } => Constant::Integer(*backing_value),
    })
}

/// Convert scalar evaluation output to the shared value representation.
pub(super) fn from_scalar(value: Constant) -> Value {
    match value {
        Constant::Integer(value) => Value::Integer(value),
        Constant::Real(value) => Value::Real(value),
        Constant::Boolean(value) => Value::Boolean(value),
        Constant::String(value) => Value::Str(value.into()),
        Constant::Unit => Value::Unit,
    }
}

/// Select scalar data for IR arithmetic and persisted interface values.
pub(crate) fn to_scalar(value: &Value) -> Option<Constant> {
    Some(match value {
        Value::Integer(value) => Constant::Integer(*value),
        Value::Real(value) => Constant::Real(*value),
        Value::Boolean(value) => Constant::Boolean(*value),
        Value::Str(value) => Constant::String(value.to_string()),
        _ => return None,
    })
}
