//! Immediate higher-order intrinsic execution for the main task.

use fpas_bytecode::{
    ArrayIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic, SourceLocation,
    Value,
};

use super::Worker;
use crate::vm::VmError;

impl Worker {
    pub(in crate::vm) fn execute_callback_intrinsic_sync(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        _location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let result = match intrinsic {
            Intrinsic::Array(operation) => self.array_callback(operation, arguments)?,
            Intrinsic::Dict(operation) => self.dict_callback(operation, arguments)?,
            Intrinsic::Result(operation) => self.result_callback(operation, arguments)?,
            Intrinsic::Option(operation) => self.option_callback(operation, arguments)?,
            _ => return Ok(None),
        };
        Ok(result.map(Some))
    }

    fn array_callback(
        &self,
        operation: ArrayIntrinsic,
        arguments: &[Value],
    ) -> Result<Option<Value>, VmError> {
        use ArrayIntrinsic::{All, Any, Filter, Find, FindIndex, FlatMap, ForEach, Map, Reduce};
        if !matches!(
            operation,
            Map | Filter | Reduce | Find | FindIndex | Any | All | FlatMap | ForEach
        ) {
            return Ok(None);
        }
        let values = self.array_argument(arguments.first(), "Std.Array callback")?;
        let callback_index = usize::from(matches!(operation, Reduce)) + 1;
        let callback = arguments
            .get(callback_index)
            .ok_or_else(|| self.arity_error("array callback"))?;
        let result = match operation {
            Map => Value::Array(
                values
                    .iter()
                    .map(|value| self.call_callback_sync(callback, std::slice::from_ref(value)))
                    .collect::<Result<Vec<_>, _>>()?
                    .into(),
            ),
            Filter => {
                let mut filtered = Vec::new();
                for value in values {
                    if self.callback_is_true(callback, std::slice::from_ref(value))? {
                        filtered.push(value.clone());
                    }
                }
                Value::Array(filtered.into())
            }
            Reduce => {
                let mut accumulator = arguments
                    .get(1)
                    .ok_or_else(|| self.arity_error("Std.Array.Reduce"))?
                    .clone();
                for value in values {
                    let arguments = [accumulator, value.clone()];
                    accumulator = self.call_callback_sync(callback, &arguments)?;
                }
                accumulator
            }
            Find => {
                let mut found = Value::OptionNone;
                for value in values {
                    if self.callback_is_true(callback, std::slice::from_ref(value))? {
                        found = Value::OptionSome(Box::new(value.clone()));
                        break;
                    }
                }
                found
            }
            FindIndex => {
                let mut found = -1;
                for (index, value) in values.iter().enumerate() {
                    if self.callback_is_true(callback, std::slice::from_ref(value))? {
                        found = index as i64;
                        break;
                    }
                }
                Value::Integer(found)
            }
            Any => {
                let mut matched = false;
                for value in values {
                    if self.callback_is_true(callback, std::slice::from_ref(value))? {
                        matched = true;
                        break;
                    }
                }
                Value::Boolean(matched)
            }
            All => {
                let mut matched = true;
                for value in values {
                    if !self.callback_is_true(callback, std::slice::from_ref(value))? {
                        matched = false;
                        break;
                    }
                }
                Value::Boolean(matched)
            }
            FlatMap => {
                let mut flattened = Vec::new();
                for value in values {
                    match self.call_callback_sync(callback, std::slice::from_ref(value))? {
                        Value::Array(inner) => flattened.extend(inner.iter().cloned()),
                        other => flattened.push(other),
                    }
                }
                Value::Array(flattened.into())
            }
            ForEach => {
                for value in values {
                    self.call_callback_sync(callback, std::slice::from_ref(value))?;
                }
                Value::Unit
            }
            _ => unreachable!("higher-order array operation was filtered above"),
        };
        Ok(Some(result))
    }

    fn dict_callback(
        &self,
        operation: DictIntrinsic,
        arguments: &[Value],
    ) -> Result<Option<Value>, VmError> {
        if !matches!(operation, DictIntrinsic::Map | DictIntrinsic::Filter) {
            return Ok(None);
        }
        let Value::Dict(entries) = arguments
            .first()
            .ok_or_else(|| self.arity_error("dict callback"))?
        else {
            return Err(self.callback_type_error("dict", arguments.first()));
        };
        let callback = arguments
            .get(1)
            .ok_or_else(|| self.arity_error("dict callback"))?;
        let mut result = Vec::with_capacity(entries.len());
        for (key, value) in entries.iter() {
            match operation {
                DictIntrinsic::Map => result.push((
                    key.clone(),
                    self.call_callback_sync(callback, vec![value.clone()])?,
                )),
                DictIntrinsic::Filter => {
                    let arguments = [key.clone(), value.clone()];
                    if self.callback_is_true(callback, &arguments)? {
                        result.push((key.clone(), value.clone()));
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(Some(Value::dict(result)))
    }

    fn result_callback(
        &self,
        operation: ResultIntrinsic,
        arguments: &[Value],
    ) -> Result<Option<Value>, VmError> {
        if !matches!(
            operation,
            ResultIntrinsic::Map | ResultIntrinsic::AndThen | ResultIntrinsic::OrElse
        ) {
            return Ok(None);
        }
        let value = arguments
            .first()
            .ok_or_else(|| self.arity_error("result callback"))?;
        let callback = arguments
            .get(1)
            .ok_or_else(|| self.arity_error("result callback"))?;
        let result = match (operation, value) {
            (ResultIntrinsic::Map, Value::ResultOk(inner)) => Value::ResultOk(Box::new(
                self.call_callback_sync(callback, vec![(**inner).clone()])?,
            )),
            (ResultIntrinsic::AndThen, Value::ResultOk(inner)) => {
                self.call_callback_sync(callback, vec![(**inner).clone()])?
            }
            (ResultIntrinsic::OrElse, Value::ResultError(inner)) => {
                self.call_callback_sync(callback, vec![(**inner).clone()])?
            }
            (_, Value::ResultOk(_) | Value::ResultError(_)) => value.clone(),
            _ => return Err(self.callback_type_error("result", Some(value))),
        };
        Ok(Some(result))
    }

    fn option_callback(
        &self,
        operation: OptionIntrinsic,
        arguments: &[Value],
    ) -> Result<Option<Value>, VmError> {
        if !matches!(
            operation,
            OptionIntrinsic::Map | OptionIntrinsic::AndThen | OptionIntrinsic::OrElse
        ) {
            return Ok(None);
        }
        let value = arguments
            .first()
            .ok_or_else(|| self.arity_error("option callback"))?;
        let callback = arguments
            .get(1)
            .ok_or_else(|| self.arity_error("option callback"))?;
        let result = match (operation, value) {
            (OptionIntrinsic::Map, Value::OptionSome(inner)) => Value::OptionSome(Box::new(
                self.call_callback_sync(callback, vec![(**inner).clone()])?,
            )),
            (OptionIntrinsic::AndThen, Value::OptionSome(inner)) => {
                self.call_callback_sync(callback, vec![(**inner).clone()])?
            }
            (OptionIntrinsic::OrElse, Value::OptionNone) => {
                self.call_callback_sync(callback, Vec::new())?
            }
            (_, Value::OptionSome(_) | Value::OptionNone) => value.clone(),
            _ => return Err(self.callback_type_error("option", Some(value))),
        };
        Ok(Some(result))
    }

    fn callback_is_true(&self, callback: &Value, arguments: &[Value]) -> Result<bool, VmError> {
        match self.call_callback_sync(callback, arguments)? {
            Value::Boolean(value) => Ok(value),
            other => Err(self.callback_type_error("boolean callback result", Some(&other))),
        }
    }
}
