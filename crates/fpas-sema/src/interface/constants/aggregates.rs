//! Aggregate construction and comparisons for checked static evaluation.
//!
//! **Documentation:** `docs/pascal/language/basics/constants.md`.

use std::sync::Arc;

use fpas_bytecode::{RecordTypeId, RuntimeRecordLayout, SharedRecord, Value};
use fpas_parser::{Designator, Expr};

use super::{StaticConstants, StaticEvaluationError, StaticRecord};

impl StaticConstants {
    /// Evaluate every eager component and retain data needed by enclosing comparisons.
    pub(super) fn evaluate_aggregate(
        &self,
        expression: &Expr,
        lookup: &impl Fn(&Designator) -> Option<Value>,
        record: &impl Fn(&Expr) -> Option<StaticRecord>,
    ) -> Result<Option<Value>, StaticEvaluationError> {
        let evaluate = |expression: &Expr| self.evaluate_with(expression, lookup, record);
        Ok(match expression {
            Expr::ArrayLiteral(elements, _) => {
                let values = elements
                    .iter()
                    .map(evaluate)
                    .collect::<Result<Vec<_>, _>>()?;
                values
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .map(|values| Value::Array(values.into()))
            }
            Expr::DictLiteral(pairs, _) => {
                let values = pairs
                    .iter()
                    .map(|(key, value)| {
                        let key = evaluate(key)?;
                        let value = evaluate(value)?;
                        Ok(key.zip(value))
                    })
                    .collect::<Result<Vec<_>, StaticEvaluationError>>()?;
                values
                    .into_iter()
                    .collect::<Option<Vec<_>>>()
                    .map(Value::dict)
            }
            Expr::OptionSome(inner, _) => evaluate(inner)?.map(Value::option_some),
            Expr::OptionNone(_) => Some(Value::OptionNone),
            Expr::ResultOk(inner, _) => evaluate(inner)?.map(Value::result_ok),
            Expr::ResultError(inner, _) => evaluate(inner)?.map(Value::result_error),
            Expr::RecordConstruction { fields, .. } | Expr::RecordUpdate { fields, .. } => {
                let base = if let Expr::RecordUpdate { base, .. } = expression {
                    evaluate(base)?
                } else {
                    None
                };
                let mut supplied = Vec::new();
                for field in fields {
                    supplied.push((field.name.to_ascii_lowercase(), evaluate(&field.value)?));
                }
                if let Expr::RecordUpdate { .. } = expression {
                    let Some(Value::Record(mut base)) = base else {
                        return Ok(None);
                    };
                    for (name, value) in supplied {
                        let Some(index) = base
                            .body()
                            .layout
                            .fields
                            .iter()
                            .position(|field| field == &name)
                        else {
                            return Ok(None);
                        };
                        let Some(value) = value else {
                            return Ok(None);
                        };
                        base.values_mut()[index] = value;
                    }
                    Some(Value::Record(base))
                } else {
                    self.construct_record(expression, supplied, &evaluate, record)?
                }
            }
            Expr::Call { .. } => {
                self.construct_record(expression, Vec::new(), &evaluate, record)?
            }
            _ => None,
        })
    }

    fn construct_record(
        &self,
        expression: &Expr,
        mut supplied: Vec<(String, Option<Value>)>,
        evaluate: &impl Fn(&Expr) -> Result<Option<Value>, StaticEvaluationError>,
        record: &impl Fn(&Expr) -> Option<StaticRecord>,
    ) -> Result<Option<Value>, StaticEvaluationError> {
        let Some(record) = record(expression) else {
            return Ok(None);
        };
        for (name, default) in record.defaults {
            supplied.push((name.to_ascii_lowercase(), evaluate(&default)?));
        }
        let names = record
            .fields
            .into_iter()
            .map(|name| name.to_ascii_lowercase())
            .collect::<Vec<_>>();
        let values = names
            .iter()
            .map(|name| {
                supplied
                    .iter()
                    .find(|(field, _)| field == name)
                    .and_then(|(_, value)| value.clone())
            })
            .collect::<Option<Vec<_>>>();
        Ok(values.map(|values| {
            Value::Record(SharedRecord::new(
                Arc::new(RuntimeRecordLayout {
                    record: RecordTypeId::new(0),
                    type_name: String::new(),
                    fields: names,
                }),
                values,
            ))
        }))
    }
}

/// Apply membership using the same component equality as executable values.
pub(super) fn membership(needle: &Value, aggregate: &Value) -> Option<bool> {
    Some(match aggregate {
        Value::Array(values) => values.iter().any(|value| value.language_equal(needle)),
        Value::Dict(pairs) => pairs.iter().any(|(key, _)| key.language_equal(needle)),
        Value::Str(text) => match needle {
            Value::Str(value) => text.contains(value.as_ref()),
            _ => return None,
        },
        _ => return None,
    })
}
