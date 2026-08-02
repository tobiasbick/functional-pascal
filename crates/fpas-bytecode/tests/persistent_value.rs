use std::sync::{Arc, Mutex};

use fpas_bytecode::{PersistentValue, PersistentValueError, Value};

#[test]
fn supported_values_round_trip_without_losing_representation() {
    let values = [
        PersistentValue::Integer(i64::MIN),
        PersistentValue::Real((-0.0_f64).to_bits()),
        PersistentValue::Real(0x7ff8_0000_0000_0042),
        PersistentValue::Boolean(true),
        PersistentValue::String("Grüße".to_string()),
        PersistentValue::Unit,
        PersistentValue::Function {
            name: "demo.run".to_string(),
            task_bound: false,
        },
        PersistentValue::Function {
            name: "demo.bound".to_string(),
            task_bound: true,
        },
    ];

    for expected in values {
        let runtime = expected.to_value();
        assert_eq!(PersistentValue::from_value(&runtime), Ok(expected));
    }
}

#[test]
fn runtime_only_value_categories_are_rejected() {
    let cases = [
        (
            Value::enum_value("E".into(), "V".into(), Vec::new()),
            "enum",
        ),
        (Value::Array(Vec::new().into()), "array"),
        (Value::dict(Vec::new()), "dict"),
        (Value::record("R".into(), Vec::new()), "record"),
        (Value::ResultOk(Box::new(Value::Unit)), "Result.Ok"),
        (Value::ResultError(Box::new(Value::Unit)), "Result.Error"),
        (Value::OptionSome(Box::new(Value::Unit)), "Option.Some"),
        (Value::OptionNone, "Option.None"),
        (Value::Cell(Arc::new(Mutex::new(Value::Unit))), "cell"),
        (Value::Task(1), "task"),
    ];

    for (value, value_type) in cases {
        assert_eq!(
            PersistentValue::from_value(&value),
            Err(PersistentValueError::UnsupportedRuntimeValue(
                value_type.to_string()
            ))
        );
    }
}

#[test]
fn capturing_function_is_rejected() {
    let function = Value::function("demo.capture".into(), vec![Value::Integer(1)], false);

    assert_eq!(
        PersistentValue::from_value(&function),
        Err(PersistentValueError::UnsupportedRuntimeValue(
            "function".to_string()
        ))
    );
}

#[test]
fn unsupported_value_error_is_actionable() {
    let error = PersistentValueError::UnsupportedRuntimeValue("array".to_string());

    assert_eq!(
        error.to_string(),
        "runtime value of type `array` cannot be stored in compiled bytecode"
    );
}
