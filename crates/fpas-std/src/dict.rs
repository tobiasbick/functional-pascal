//! Runtime implementations for `Std.Dict.*` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/collections/dict.md`

use crate::error::StdError;
use crate::intrinsic_args::{IntrinsicCall, expect_dict, pop_dict, pop_value};
use fpas_bytecode::{DictIntrinsic, Intrinsic, SourceLocation, Value};

/// Executes dictionary operations, borrowing entries for reads.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Dict(DictIntrinsic::Length) => {
            let pairs = expect_dict(pop_value(call, location)?, location)?;
            call.push(Value::Integer(pairs.len() as i64));
        }
        Intrinsic::Dict(DictIntrinsic::ContainsKey) => {
            let key = pop_value(call, location)?;
            let pairs = expect_dict(pop_value(call, location)?, location)?;
            let found = pairs.iter().any(|(k, _)| k == key);
            call.push(Value::Boolean(found));
        }
        Intrinsic::Dict(DictIntrinsic::Keys) => {
            let pairs = expect_dict(pop_value(call, location)?, location)?;
            let keys: Vec<Value> = pairs.iter().map(|(k, _)| k.clone()).collect();
            call.push(Value::Array(keys.into()));
        }
        Intrinsic::Dict(DictIntrinsic::Values) => {
            let pairs = expect_dict(pop_value(call, location)?, location)?;
            let values: Vec<Value> = pairs.iter().map(|(_, v)| v.clone()).collect();
            call.push(Value::Array(values.into()));
        }
        Intrinsic::Dict(DictIntrinsic::Remove) => {
            let key = pop_value(call, location)?;
            let mut pairs = pop_dict(pop_value(call, location)?, location)?;
            pairs.retain(|(k, _)| k != key);
            call.push(Value::dict(pairs));
        }
        Intrinsic::Dict(DictIntrinsic::Get) => {
            let key = pop_value(call, location)?;
            let pairs = expect_dict(pop_value(call, location)?, location)?;
            let found = pairs.iter().find(|(k, _)| k == key);
            match found {
                Some((_, v)) => call.push(Value::option_some(v.clone())),
                None => call.push(Value::OptionNone),
            }
        }
        Intrinsic::Dict(DictIntrinsic::Merge) => {
            let other = pop_dict(pop_value(call, location)?, location)?;
            let mut base = pop_dict(pop_value(call, location)?, location)?;
            for (k, v) in other {
                if let Some(entry) = base.iter_mut().find(|(ek, _)| ek == &k) {
                    entry.1 = v;
                } else {
                    base.push((k, v));
                }
            }
            call.push(Value::dict(base));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_dict(intrinsic: DictIntrinsic, stack: &mut Vec<Value>) -> Result<(), StdError> {
        crate::execute_test_intrinsic(Intrinsic::Dict(intrinsic), stack, loc()).map(|_| ())
    }

    #[test]
    fn get_returns_some_for_existing_key() {
        let mut stack = vec![
            Value::dict(vec![
                (Value::Str("a".into()), Value::Integer(1)),
                (Value::Str("b".into()), Value::Integer(2)),
            ]),
            Value::Str("b".into()),
        ];
        run_dict(DictIntrinsic::Get, &mut stack).unwrap();
        assert_eq!(stack, vec![Value::option_some(Value::Integer(2))]);
    }

    #[test]
    fn contains_key_reports_presence() {
        let mut stack = vec![
            Value::dict(vec![(Value::Str("a".into()), Value::Integer(1))]),
            Value::Str("missing".into()),
        ];
        run_dict(DictIntrinsic::ContainsKey, &mut stack).unwrap();
        assert_eq!(stack, vec![Value::Boolean(false)]);
    }

    #[test]
    fn merge_overwrites_and_appends_keys() {
        let mut stack = vec![
            Value::dict(vec![
                (Value::Str("a".into()), Value::Integer(1)),
                (Value::Str("b".into()), Value::Integer(2)),
            ]),
            Value::dict(vec![
                (Value::Str("b".into()), Value::Integer(20)),
                (Value::Str("c".into()), Value::Integer(30)),
            ]),
        ];
        run_dict(DictIntrinsic::Merge, &mut stack).unwrap();
        assert_eq!(
            stack,
            vec![Value::dict(vec![
                (Value::Str("a".into()), Value::Integer(1)),
                (Value::Str("b".into()), Value::Integer(20)),
                (Value::Str("c".into()), Value::Integer(30)),
            ])]
        );
    }
    #[test]
    fn read_operations_preserve_order_managed_values_and_input() {
        let pairs = vec![
            (
                Value::Str("b".into()),
                Value::Array(vec![Value::Integer(2)].into()),
            ),
            (
                Value::Str("a".into()),
                Value::option_some(Value::Str("one".into())),
            ),
        ];
        let dictionary = Value::dict(pairs.clone());
        for (intrinsic, key, expected) in [
            (DictIntrinsic::Length, None, Value::Integer(2)),
            (
                DictIntrinsic::ContainsKey,
                Some(Value::Str("a".into())),
                Value::Boolean(true),
            ),
            (
                DictIntrinsic::Get,
                Some(Value::Str("a".into())),
                Value::option_some(pairs[1].1.clone()),
            ),
            (
                DictIntrinsic::Get,
                Some(Value::Str("missing".into())),
                Value::OptionNone,
            ),
            (
                DictIntrinsic::Keys,
                None,
                Value::Array(
                    pairs
                        .iter()
                        .map(|(key, _)| key.clone())
                        .collect::<Vec<_>>()
                        .into(),
                ),
            ),
            (
                DictIntrinsic::Values,
                None,
                Value::Array(
                    pairs
                        .iter()
                        .map(|(_, value)| value.clone())
                        .collect::<Vec<_>>()
                        .into(),
                ),
            ),
        ] {
            let mut stack = vec![dictionary.clone()];
            stack.extend(key);
            run_dict(intrinsic, &mut stack).unwrap();
            assert_eq!(stack, vec![expected]);
            assert_eq!(dictionary, Value::dict(pairs.clone()));
        }
    }
}
