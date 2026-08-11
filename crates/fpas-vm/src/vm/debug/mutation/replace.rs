//! Copy-on-write rebuilding of supported writable aggregate paths.

use fpas_bytecode::Value;

use super::super::inspection::MutationPath;
use super::super::types::{DebugErrorKind, DebugSessionError};

pub(super) fn descendant(
    mut root: Value,
    path: &[MutationPath],
    replacement: Value,
) -> Result<Value, DebugSessionError> {
    replace_at(&mut root, path, replacement)?;
    Ok(root)
}

pub(super) fn resolve<'a>(mut value: &'a Value, path: &[MutationPath]) -> Option<&'a Value> {
    for component in path {
        value = match (component, value) {
            (MutationPath::RecordField(index), Value::Record(record)) => {
                record.body().values.get(*index)?
            }
            (MutationPath::ArrayIndex(index), Value::Array(array)) => array.get(*index)?,
            (MutationPath::DictionaryValue(key), Value::Dict(dictionary)) => dictionary
                .iter()
                .find(|(candidate, _)| candidate == key)
                .map(|(_, value)| value)?,
            _ => return None,
        };
    }
    Some(value)
}

fn replace_at(
    current: &mut Value,
    path: &[MutationPath],
    replacement: Value,
) -> Result<(), DebugSessionError> {
    let Some((component, rest)) = path.split_first() else {
        *current = replacement;
        return Ok(());
    };
    let child = match (component, current) {
        (MutationPath::RecordField(index), Value::Record(record)) => record
            .values_mut()
            .get_mut(*index)
            .ok_or_else(path_unavailable)?,
        (MutationPath::ArrayIndex(index), Value::Array(array)) => {
            array.get_mut(*index).ok_or_else(path_unavailable)?
        }
        (MutationPath::DictionaryValue(key), Value::Dict(dictionary)) => dictionary
            .iter_mut()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value)
            .ok_or_else(path_unavailable)?,
        _ => return Err(path_unavailable()),
    };
    replace_at(child, rest, replacement)
}

fn path_unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug variable aggregate path changed before commit".to_string(),
        hint: "Request the variable tree again and retry the update.".to_string(),
    }
}
