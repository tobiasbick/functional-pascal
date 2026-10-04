//! FPAS data equality remains independent of representation equality.

use crate::Value;
use std::mem::ManuallyDrop;

#[test]
fn mapping_equality_matches_keys_and_values_independently_of_order() {
    let a = Value::dict(vec![
        (Value::Integer(1), Value::Integer(10)),
        (Value::Integer(2), Value::Integer(20)),
    ]);
    let b = Value::dict(vec![
        (Value::Integer(2), Value::Integer(20)),
        (Value::Integer(1), Value::Integer(10)),
    ]);
    assert!(a.language_equal(&b));
    assert_ne!(a, b);
    for different in [
        Value::dict(vec![
            (Value::Integer(1), Value::Integer(20)),
            (Value::Integer(2), Value::Integer(10)),
        ]),
        Value::dict(vec![
            (Value::Integer(3), Value::Integer(20)),
            (Value::Integer(1), Value::Integer(10)),
        ]),
        Value::dict(vec![(Value::Integer(1), Value::Integer(10))]),
    ] {
        assert!(!a.language_equal(&different));
    }
}

#[test]
fn nested_keys_are_matched_after_failed_candidates() {
    let key = |n| Value::Array(vec![Value::Integer(n)].into());
    let a = Value::dict(vec![
        (key(1), Value::Integer(10)),
        (key(2), Value::Integer(20)),
    ]);
    let b = Value::dict(vec![
        (key(2), Value::Integer(20)),
        (key(1), Value::Integer(10)),
    ]);
    assert!(a.language_equal(&b));
    assert!(Value::dict(vec![]).language_equal(&Value::dict(vec![])));
}

#[test]
fn ieee_real_equality_applies_to_nested_values_and_keys() {
    let nan = Value::Real(f64::from_bits(0x7ff8_0000_0000_0001));
    assert_eq!(nan, nan.clone());
    assert!(!nan.language_equal(&nan));
    let a = Value::option_some(Value::Array(vec![nan.clone()].into()));
    assert!(!a.language_equal(&a));
    let a = Value::dict(vec![(Value::Integer(1), nan.clone())]);
    assert!(!a.language_equal(&a));
    let a = Value::dict(vec![(nan, Value::Integer(1))]);
    assert!(!a.language_equal(&a));
    let a = Value::dict(vec![(Value::Real(0.0), Value::Real(-0.0))]);
    let b = Value::dict(vec![(Value::Real(-0.0), Value::Real(0.0))]);
    assert!(a.language_equal(&b));
}

#[test]
fn resource_and_task_identities_do_not_compare_as_data() {
    for value in [
        Value::OpaqueHandle(1),
        Value::Task(1),
        Value::option_some(Value::OpaqueHandle(1)),
    ] {
        assert_eq!(value, value.clone());
        assert!(!value.language_equal(&value));
    }
}

#[test]
fn deeply_nested_dictionary_keys_use_an_explicit_comparison_stack() {
    let mut a = Value::Integer(1);
    let mut b = Value::Integer(1);
    for _ in 0..10_000 {
        a = Value::Array(vec![a].into());
        b = Value::Array(vec![b].into());
    }
    // Keep recursive storage destruction out of this comparison-depth regression.
    let a = ManuallyDrop::new(Value::dict(vec![(a, Value::Integer(2))]));
    let b = ManuallyDrop::new(Value::dict(vec![(b, Value::Integer(2))]));
    assert!(a.language_equal(&b));
}
