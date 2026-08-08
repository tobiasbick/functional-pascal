//! Runtime implementations for `Std.Dict.*` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/collections/dict.md`

use crate::error::StdError;
use crate::intrinsic_args::{IntrinsicCall, pop_dict, pop_value};
use fpas_bytecode::{DictIntrinsic, Intrinsic, SourceLocation, Value};

pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Dict(DictIntrinsic::Length) => {
            let pairs = pop_dict(pop_value(call, location)?, location)?;
            call.push(Value::Integer(pairs.len() as i64));
        }
        Intrinsic::Dict(DictIntrinsic::ContainsKey) => {
            let key = pop_value(call, location)?;
            let pairs = pop_dict(pop_value(call, location)?, location)?;
            let found = pairs.iter().any(|(k, _)| k == key);
            call.push(Value::Boolean(found));
        }
        Intrinsic::Dict(DictIntrinsic::Keys) => {
            let pairs = pop_dict(pop_value(call, location)?, location)?;
            let keys: Vec<Value> = pairs.into_iter().map(|(k, _)| k).collect();
            call.push(Value::Array(keys.into()));
        }
        Intrinsic::Dict(DictIntrinsic::Values) => {
            let pairs = pop_dict(pop_value(call, location)?, location)?;
            let values: Vec<Value> = pairs.into_iter().map(|(_, v)| v).collect();
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
            let pairs = pop_dict(pop_value(call, location)?, location)?;
            let found = pairs.into_iter().find(|(k, _)| k == key);
            match found {
                Some((_, v)) => call.push(Value::OptionSome(Box::new(v))),
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
        crate::run_intrinsic(Intrinsic::Dict(intrinsic), stack, loc()).map(|_| ())
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
        assert_eq!(stack, vec![Value::OptionSome(Box::new(Value::Integer(2)))]);
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
}
