//! Binding-first call resolution for qualified names and record methods.
//! See `docs/pascal/language/functions/parameters.md`.

use fpas_bytecode::Value;

use super::model::DebugCallTarget;
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::value_ops;

/// Resolves visible bindings before interpreting a name as an executable callable.
pub(super) fn resolve(
    name: &str,
    resolve_binding: &mut impl FnMut(&str) -> Result<Value, DebugSessionError>,
) -> Result<DebugCallTarget, DebugSessionError> {
    let mut parts = name.split('.');
    let root = parts.next().unwrap_or(name);
    let mut value = match resolve_binding(root) {
        Ok(value) => value,
        Err(error) if error.kind == DebugErrorKind::UnknownName => {
            return Ok(DebugCallTarget::Named(name.into()));
        }
        Err(error) => return Err(error),
    };
    let members = parts.collect::<Vec<_>>();
    for (index, member) in members.iter().enumerate() {
        if index + 1 == members.len()
            && let Value::Record(record) = &value
            && !record
                .body()
                .layout
                .fields
                .iter()
                .any(|field| field.eq_ignore_ascii_case(member))
        {
            return Ok(DebugCallTarget::Method {
                receiver: value,
                name: (*member).into(),
            });
        }
        value = value_ops::field(&value, member).map_err(|error| DebugSessionError {
            kind: DebugErrorKind::EvaluationType,
            message: error.message,
            hint: error.hint,
        })?;
    }
    Ok(DebugCallTarget::Value(value))
}
