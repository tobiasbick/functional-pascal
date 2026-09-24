//! Construction of resumable higher-order operation state.

use fpas_bytecode::{
    ArrayIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic, StrIntrinsic, Value,
};

use super::Worker;
use super::arguments::CallbackArguments;
use super::operation::{
    CallbackOperation, Cursor, Dictionary, DictionaryReduction, Sequence, SingleWrapper,
};
use super::string::{StringReduction, StringSequence};
use crate::vm::VmError;

/// Return a pass-through result when the callback branch is inactive.
pub(super) fn inactive_result(
    worker: &Worker,
    intrinsic: Intrinsic,
    arguments: &[Value],
) -> Result<Option<Value>, VmError> {
    match intrinsic {
        Intrinsic::Result(operation) if is_result_callback(operation) => {
            let value = arguments
                .first()
                .ok_or_else(|| worker.arity_error("result callback"))?;
            require_callback_argument(worker, arguments, "result callback")?;
            match (operation, value) {
                (ResultIntrinsic::Map | ResultIntrinsic::AndThen, Value::ResultError(_))
                | (ResultIntrinsic::OrElse, Value::ResultOk(_)) => Ok(Some(value.clone())),
                (_, Value::ResultOk(_) | Value::ResultError(_)) => Ok(None),
                _ => Err(worker.callback_type_error("result", Some(value))),
            }
        }
        Intrinsic::Option(operation) if is_option_callback(operation) => {
            let value = arguments
                .first()
                .ok_or_else(|| worker.arity_error("option callback"))?;
            require_callback_argument(worker, arguments, "option callback")?;
            match (operation, value) {
                (OptionIntrinsic::Map | OptionIntrinsic::AndThen, Value::OptionNone)
                | (OptionIntrinsic::OrElse, Value::OptionSome(_)) => Ok(Some(value.clone())),
                (_, Value::OptionSome(_) | Value::OptionNone) => Ok(None),
                _ => Err(worker.callback_type_error("option", Some(value))),
            }
        }
        _ => Ok(None),
    }
}

/// Build state for a supported callback intrinsic that must invoke its callback.
pub(super) fn operation(
    worker: &Worker,
    intrinsic: Intrinsic,
    arguments: &[Value],
) -> Result<Option<(Value, CallbackOperation)>, VmError> {
    match intrinsic {
        Intrinsic::Array(operation) => array_operation(worker, operation, arguments),
        Intrinsic::Dict(operation) => dict_operation(worker, operation, arguments),
        Intrinsic::Str(operation) => str_operation(worker, operation, arguments),
        Intrinsic::Result(operation) => result_operation(worker, operation, arguments),
        Intrinsic::Option(operation) => option_operation(worker, operation, arguments),
        _ => Ok(None),
    }
}

fn array_operation(
    worker: &Worker,
    operation: ArrayIntrinsic,
    arguments: &[Value],
) -> Result<Option<(Value, CallbackOperation)>, VmError> {
    use ArrayIntrinsic::{All, Any, Filter, Find, FindIndex, FlatMap, ForEach, Map, Reduce};
    if !matches!(
        operation,
        Map | Filter | Reduce | Find | FindIndex | Any | All | FlatMap | ForEach
    ) {
        return Ok(None);
    }
    let values = worker
        .array_argument(arguments.first(), "Std.Arrays callback")?
        .to_vec();
    let callback_index = usize::from(matches!(operation, Reduce)) + 1;
    let callback = arguments
        .get(callback_index)
        .ok_or_else(|| worker.arity_error("array callback"))?
        .clone();
    let operation = match operation {
        Map => CallbackOperation::ArrayMap(Sequence::new(values)),
        Filter => CallbackOperation::ArrayFilter(Sequence::new(values)),
        Reduce => CallbackOperation::ArrayReduce {
            values,
            next: 0,
            accumulator: arguments
                .get(1)
                .ok_or_else(|| worker.arity_error("Std.Arrays.Reduce"))?
                .clone(),
        },
        Find => CallbackOperation::ArrayFind(Cursor::new(values)),
        FindIndex => CallbackOperation::ArrayFindIndex(Cursor::new(values)),
        Any => CallbackOperation::ArrayAny(Cursor::new(values)),
        All => CallbackOperation::ArrayAll(Cursor::new(values)),
        FlatMap => CallbackOperation::ArrayFlatMap(Sequence::new(values)),
        ForEach => CallbackOperation::ArrayForEach(Cursor::new(values)),
        _ => unreachable!(),
    };
    Ok(Some((callback, operation)))
}

fn dict_operation(
    worker: &Worker,
    operation: DictIntrinsic,
    arguments: &[Value],
) -> Result<Option<(Value, CallbackOperation)>, VmError> {
    if !matches!(
        operation,
        DictIntrinsic::Map | DictIntrinsic::Filter | DictIntrinsic::Reduce
    ) {
        return Ok(None);
    }
    let Value::Dict(entries) = arguments
        .first()
        .ok_or_else(|| worker.arity_error("dict callback"))?
    else {
        return Err(worker.callback_type_error("dict", arguments.first()));
    };
    let callback_index = if operation == DictIntrinsic::Reduce {
        2
    } else {
        1
    };
    let callback = arguments
        .get(callback_index)
        .ok_or_else(|| worker.arity_error("dict callback"))?
        .clone();
    let operation = match operation {
        DictIntrinsic::Map => {
            CallbackOperation::DictMap(Dictionary::new(entries.iter().cloned().collect()))
        }
        DictIntrinsic::Filter => {
            CallbackOperation::DictFilter(Dictionary::new(entries.iter().cloned().collect()))
        }
        DictIntrinsic::Reduce => CallbackOperation::DictReduce(DictionaryReduction::new(
            entries.iter().cloned().collect(),
            arguments
                .get(1)
                .ok_or_else(|| worker.arity_error("Std.Dictionaries.Reduce"))?
                .clone(),
        )),
        _ => unreachable!(),
    };
    Ok(Some((callback, operation)))
}

fn str_operation(
    worker: &Worker,
    operation: StrIntrinsic,
    arguments: &[Value],
) -> Result<Option<(Value, CallbackOperation)>, VmError> {
    if !matches!(
        operation,
        StrIntrinsic::Map | StrIntrinsic::Filter | StrIntrinsic::Reduce
    ) {
        return Ok(None);
    }
    let Value::Str(input) = arguments
        .first()
        .ok_or_else(|| worker.arity_error("string callback"))?
    else {
        return Err(worker.callback_type_error("string", arguments.first()));
    };
    let callback_index = if operation == StrIntrinsic::Reduce {
        2
    } else {
        1
    };
    let callback = arguments
        .get(callback_index)
        .ok_or_else(|| worker.arity_error("string callback"))?
        .clone();
    let operation = match operation {
        StrIntrinsic::Map => CallbackOperation::StringMap(StringSequence::new(input)),
        StrIntrinsic::Filter => CallbackOperation::StringFilter(StringSequence::new(input)),
        StrIntrinsic::Reduce => CallbackOperation::StringReduce(StringReduction::new(
            input,
            arguments
                .get(1)
                .ok_or_else(|| worker.arity_error("Std.Str.Reduce"))?
                .clone(),
        )),
        _ => unreachable!(),
    };
    Ok(Some((callback, operation)))
}

fn result_operation(
    worker: &Worker,
    operation: ResultIntrinsic,
    arguments: &[Value],
) -> Result<Option<(Value, CallbackOperation)>, VmError> {
    if !is_result_callback(operation) {
        return Ok(None);
    }
    let value = arguments
        .first()
        .ok_or_else(|| worker.arity_error("result callback"))?;
    let callback = require_callback_argument(worker, arguments, "result callback")?.clone();
    let (arguments, wrapper) = match (operation, value) {
        (ResultIntrinsic::Map, Value::ResultOk(inner)) => (
            CallbackArguments::one((**inner).clone()),
            SingleWrapper::ResultOk,
        ),
        (ResultIntrinsic::AndThen, Value::ResultOk(inner))
        | (ResultIntrinsic::OrElse, Value::ResultError(inner)) => (
            CallbackArguments::one((**inner).clone()),
            SingleWrapper::Direct,
        ),
        (_, Value::ResultOk(_) | Value::ResultError(_)) => return Ok(None),
        _ => return Err(worker.callback_type_error("result", Some(value))),
    };
    Ok(Some((
        callback,
        CallbackOperation::Single {
            arguments: Some(arguments),
            wrapper,
        },
    )))
}

fn option_operation(
    worker: &Worker,
    operation: OptionIntrinsic,
    arguments: &[Value],
) -> Result<Option<(Value, CallbackOperation)>, VmError> {
    if !is_option_callback(operation) {
        return Ok(None);
    }
    let value = arguments
        .first()
        .ok_or_else(|| worker.arity_error("option callback"))?;
    let callback = require_callback_argument(worker, arguments, "option callback")?.clone();
    let (arguments, wrapper) = match (operation, value) {
        (OptionIntrinsic::Map, Value::OptionSome(inner)) => (
            CallbackArguments::one((**inner).clone()),
            SingleWrapper::OptionSome,
        ),
        (OptionIntrinsic::AndThen, Value::OptionSome(inner)) => (
            CallbackArguments::one((**inner).clone()),
            SingleWrapper::Direct,
        ),
        (OptionIntrinsic::OrElse, Value::OptionNone) => {
            (CallbackArguments::empty(), SingleWrapper::Direct)
        }
        (_, Value::OptionSome(_) | Value::OptionNone) => return Ok(None),
        _ => return Err(worker.callback_type_error("option", Some(value))),
    };
    Ok(Some((
        callback,
        CallbackOperation::Single {
            arguments: Some(arguments),
            wrapper,
        },
    )))
}

fn require_callback_argument<'a>(
    worker: &Worker,
    arguments: &'a [Value],
    context: &str,
) -> Result<&'a Value, VmError> {
    arguments.get(1).ok_or_else(|| worker.arity_error(context))
}

fn is_result_callback(operation: ResultIntrinsic) -> bool {
    matches!(
        operation,
        ResultIntrinsic::Map | ResultIntrinsic::AndThen | ResultIntrinsic::OrElse
    )
}

fn is_option_callback(operation: OptionIntrinsic) -> bool {
    matches!(
        operation,
        OptionIntrinsic::Map | OptionIntrinsic::AndThen | OptionIntrinsic::OrElse
    )
}
