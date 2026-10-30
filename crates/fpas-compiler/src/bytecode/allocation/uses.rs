//! Visit IR operands without temporary collections.

use fpas_ir::{Operation, Terminator, ValueId};

/// Visit every value read by `operation`, preserving operand order and repetitions.
pub(super) fn operation_values(operation: &Operation, mut visit: impl FnMut(ValueId)) {
    match operation {
        Operation::Const(_)
        | Operation::ReadLocal(_)
        | Operation::ArrayPop { .. }
        | Operation::LoadGlobal(_)
        | Operation::MakeGlobalReference(_)
        | Operation::MakeNone
        | Operation::Yield
        | Operation::EnterTaskScope
        | Operation::ExitTaskScope
        | Operation::CancelTaskScope => {}
        Operation::WriteLocal { value, .. }
        | Operation::StoreGlobal { value, .. }
        | Operation::MakeCell(value)
        | Operation::CellRead(value)
        | Operation::MakeCellReference(value)
        | Operation::ReferenceRead(value)
        | Operation::ReferenceField {
            reference: value, ..
        } => visit(*value),
        Operation::ReferenceElement { reference, index } => {
            visit(*reference);
            visit(*index);
        }
        Operation::ReferenceWrite { reference, value } => {
            visit(*reference);
            visit(*value);
        }
        Operation::MakeOk(value)
        | Operation::MakeError(value)
        | Operation::MakeSome(value)
        | Operation::IsResultOk(value)
        | Operation::IsOptionSome(value)
        | Operation::UnwrapOk(value)
        | Operation::UnwrapError(value)
        | Operation::UnwrapSome(value) => visit(*value),
        Operation::MakeArray(values) => values.iter().copied().for_each(visit),
        Operation::ArrayPush { value, .. } => visit(*value),
        Operation::MakeDictionary(pairs) => {
            for (key, value) in pairs {
                visit(*key);
                visit(*value);
            }
        }
        Operation::IndexGet { collection, index } => {
            visit(*collection);
            visit(*index);
        }
        Operation::StoreLocalIndex { index, value, .. } => {
            visit(*index);
            visit(*value);
        }
        Operation::IndexSet {
            collection,
            index,
            value,
        } => {
            visit(*collection);
            visit(*index);
            visit(*value);
        }
        Operation::StoreGlobalIndexPath {
            root,
            indexes,
            value,
            ..
        } => {
            visit(*root);
            indexes.iter().copied().for_each(&mut visit);
            visit(*value);
        }
        Operation::Contains { value, collection } => {
            visit(*value);
            visit(*collection);
        }
        Operation::Binary { left, right, .. } => {
            visit(*left);
            visit(*right);
        }
        Operation::Unary { operand, .. } => visit(*operand),
        Operation::CallDirect { arguments, .. } | Operation::Intrinsic { arguments, .. } => {
            arguments.iter().copied().for_each(visit)
        }
        Operation::CallValue { callee, arguments } | Operation::SpawnTask { callee, arguments } => {
            visit(*callee);
            arguments.iter().copied().for_each(visit);
        }
        Operation::MakeRecord { fields, .. }
        | Operation::MakeEnum { fields, .. }
        | Operation::MakeClosure {
            captures: fields, ..
        } => fields.iter().copied().for_each(visit),
        Operation::LoadField { record, .. } => visit(*record),
        Operation::UpdateRecord { record, fields, .. } => {
            visit(*record);
            fields.iter().map(|(_, value)| *value).for_each(visit);
        }
        Operation::StoreField { record, value, .. }
        | Operation::CellWrite {
            cell: record,
            value,
        } => {
            visit(*record);
            visit(*value);
        }
        Operation::TestVariant { value, .. } => visit(*value),
        Operation::LoadEnumField { value, .. } => visit(*value),
    }
}

/// Visit terminator inputs, including block-target arguments.
pub(super) fn terminator_values(terminator: &Terminator, mut visit: impl FnMut(ValueId)) {
    match terminator {
        Terminator::Branch {
            condition,
            then_target,
            else_target,
        } => {
            visit(*condition);
            then_target.arguments.iter().copied().for_each(&mut visit);
            else_target.arguments.iter().copied().for_each(visit);
        }
        Terminator::Jump(target) => target.arguments.iter().copied().for_each(visit),
        Terminator::ForLoop {
            body_target,
            after_target,
            ..
        } => {
            body_target.arguments.iter().copied().for_each(&mut visit);
            after_target.arguments.iter().copied().for_each(visit);
        }
        Terminator::Return(value) => value.iter().copied().for_each(visit),
        Terminator::Panic(value) => visit(*value),
    }
}
