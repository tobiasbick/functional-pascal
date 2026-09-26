//! Values read by IR operations and terminators.

use fpas_ir::{Operation, Terminator, ValueId};

/// Values read by `operation`, in operand order.
pub(super) fn operation_values(operation: &Operation) -> Vec<ValueId> {
    match operation {
        Operation::Const(_)
        | Operation::ReadLocal(_)
        | Operation::ArrayPop { .. }
        | Operation::LoadGlobal(_)
        | Operation::MakeNone
        | Operation::Yield => Vec::new(),
        Operation::WriteLocal { value, .. }
        | Operation::StoreGlobal { value, .. }
        | Operation::MakeCell(value)
        | Operation::CellRead(value) => vec![*value],
        Operation::MakeOk(value)
        | Operation::MakeError(value)
        | Operation::MakeSome(value)
        | Operation::IsResultOk(value)
        | Operation::IsOptionSome(value)
        | Operation::UnwrapOk(value)
        | Operation::UnwrapError(value)
        | Operation::UnwrapSome(value) => vec![*value],
        Operation::MakeArray(values) => values.clone(),
        Operation::ArrayPush { value, .. } => vec![*value],
        Operation::MakeDictionary(pairs) => pairs
            .iter()
            .flat_map(|(key, value)| [*key, *value])
            .collect(),
        Operation::IndexGet { collection, index } => vec![*collection, *index],
        Operation::StoreLocalIndex { index, value, .. } => vec![*index, *value],
        Operation::IndexSet {
            collection,
            index,
            value,
        } => vec![*collection, *index, *value],
        Operation::StoreGlobalIndexPath {
            root,
            indexes,
            value,
            ..
        } => std::iter::once(*root)
            .chain(indexes.iter().copied())
            .chain(std::iter::once(*value))
            .collect(),
        Operation::Contains { value, collection } => vec![*value, *collection],
        Operation::Binary { left, right, .. } => vec![*left, *right],
        Operation::Unary { operand, .. } => vec![*operand],
        Operation::CallDirect { arguments, .. } | Operation::Intrinsic { arguments, .. } => {
            arguments.clone()
        }
        Operation::CallValue { callee, arguments }
        | Operation::SpawnTask { callee, arguments }
        | Operation::SpawnDetachedTask { callee, arguments } => {
            let mut values = vec![*callee];
            values.extend(arguments.iter().copied());
            values
        }
        Operation::MakeRecord { fields, .. }
        | Operation::MakeEnum { fields, .. }
        | Operation::MakeClosure {
            captures: fields, ..
        } => fields.clone(),
        Operation::LoadField { record, .. } => vec![*record],
        Operation::UpdateRecord { record, fields, .. } => std::iter::once(*record)
            .chain(fields.iter().map(|(_, value)| *value))
            .collect(),
        Operation::StoreField { record, value, .. }
        | Operation::CellWrite {
            cell: record,
            value,
        } => vec![*record, *value],
        Operation::TestVariant { value, .. } => vec![*value],
        Operation::LoadEnumField { value, .. } => vec![*value],
    }
}

/// Values read by `terminator`, including block-target arguments.
pub(super) fn terminator_values(terminator: &Terminator) -> Vec<ValueId> {
    match terminator {
        Terminator::Branch {
            condition,
            then_target,
            else_target,
        } => {
            let mut values = vec![*condition];
            values.extend(then_target.arguments.iter().copied());
            values.extend(else_target.arguments.iter().copied());
            values
        }
        Terminator::Jump(target) => target.arguments.clone(),
        Terminator::ForLoop {
            body_target,
            after_target,
            ..
        } => {
            let mut values = Vec::new();
            values.extend(body_target.arguments.iter().copied());
            values.extend(after_target.arguments.iter().copied());
            values
        }
        Terminator::Return(value) => value.iter().copied().collect(),
        Terminator::Panic(value) => vec![*value],
    }
}
