//! Deterministic lowest-free-register linear-scan allocation.

use std::collections::{BTreeMap, BTreeSet};

use fpas_bytecode::Register;
use fpas_ir::{Function, LocalId, Operation, Terminator, ValueId};

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) struct Allocation {
    locals: BTreeMap<LocalId, Register>,
    values: BTreeMap<ValueId, Register>,
    call_window: Register,
    pub register_count: u16,
}

impl Allocation {
    pub fn build(function: &Function) -> Result<Self, CompileError> {
        let mut locals = BTreeMap::new();
        let mut values = BTreeMap::new();
        for (index, parameter) in function.parameters.iter().enumerate() {
            let register =
                Register::try_from_index(index).map_err(|error| limit_error(&error.to_string()))?;
            values.insert(parameter.id, register);
        }
        let capture_locals = function
            .locals
            .iter()
            .filter(|local| local.capture.is_some())
            .collect::<Vec<_>>();
        if capture_locals.len() != function.captures.len() {
            return Err(limit_error(
                "capture declarations must have one ordered capture local",
            ));
        }
        let mut next_fixed = function.parameters.len();
        for local in capture_locals {
            let register = Register::try_from_index(next_fixed)
                .map_err(|error| limit_error(&error.to_string()))?;
            locals.insert(local.id, register);
            next_fixed = next_fixed.saturating_add(1);
        }
        for local in function
            .locals
            .iter()
            .filter(|local| local.capture.is_none())
        {
            let register = Register::try_from_index(next_fixed)
                .map_err(|error| limit_error(&error.to_string()))?;
            locals.insert(local.id, register);
            next_fixed = next_fixed.saturating_add(1);
        }

        let last_uses = last_uses(function);
        let coalesced_writes = coalesced_local_writes(function);
        let mut active: Vec<(ValueId, usize, Register)> = Vec::new();
        let mut position = 0_usize;
        let mut high_water = next_fixed;
        for block in &function.blocks {
            if !block.parameters.is_empty() {
                return Err(limit_error(
                    "register allocation received unsupported block parameters",
                ));
            }
            for instruction in &block.instructions {
                active.retain(|(_, last_use, _)| *last_use >= position);
                if let Some(result) = instruction.result {
                    let register = if let Some(local) = coalesced_writes.get(&result.id) {
                        locals.get(local).copied().ok_or_else(|| {
                            limit_error(&format!(
                                "coalesced local {} has no allocated register",
                                local.get()
                            ))
                        })?
                    } else {
                        let used: BTreeSet<u16> = active
                            .iter()
                            .map(|(_, _, register)| register.get())
                            .chain(locals.values().map(|register| register.get()))
                            .collect();
                        let register = lowest_free(next_fixed, &used)?;
                        high_water = high_water.max(usize::from(register.get()) + 1);
                        let last_use = last_uses.get(&result.id).copied().unwrap_or(position);
                        active.push((result.id, last_use, register));
                        register
                    };
                    values.insert(result.id, register);
                }
                position = position.saturating_add(1);
            }
            position = position.saturating_add(1);
        }
        let call_window = Register::try_from_index(high_water)
            .map_err(|error| limit_error(&error.to_string()))?;
        let window_size = largest_window(function);
        let register_count = u16::try_from(high_water.saturating_add(window_size))
            .map_err(|_| limit_error("register count exceeds the portable u16 frame limit"))?;
        Ok(Self {
            locals,
            values,
            call_window,
            register_count,
        })
    }

    pub fn local(&self, id: LocalId) -> Result<Register, CompileError> {
        self.locals
            .get(&id)
            .copied()
            .ok_or_else(|| limit_error(&format!("local {} has no allocated register", id.get())))
    }

    pub fn value(&self, id: ValueId) -> Result<Register, CompileError> {
        self.values
            .get(&id)
            .copied()
            .ok_or_else(|| limit_error(&format!("value {} has no allocated register", id.get())))
    }

    pub fn call_window(&self) -> Register {
        self.call_window
    }
}

fn coalesced_local_writes(function: &Function) -> BTreeMap<ValueId, LocalId> {
    let initializer_writes = function
        .debug
        .bindings
        .iter()
        .filter_map(|binding| binding.initializer)
        .map(|location| (location.block, location.instruction))
        .collect::<BTreeSet<_>>();
    let mut use_counts = BTreeMap::<ValueId, usize>::new();
    for block in &function.blocks {
        for instruction in &block.instructions {
            for value in operation_values(&instruction.operation) {
                *use_counts.entry(value).or_default() += 1;
            }
        }
        if let Some(terminator) = block.terminators.first() {
            for value in terminator_values(terminator) {
                *use_counts.entry(value).or_default() += 1;
            }
        }
    }

    let mut writes = BTreeMap::new();
    for block in &function.blocks {
        for (index, pair) in block.instructions.windows(2).enumerate() {
            let Some(result) = pair[0].result else {
                continue;
            };
            let Operation::WriteLocal { value, local } = &pair[1].operation else {
                continue;
            };
            if initializer_writes.contains(&(block.id, index.saturating_add(1))) {
                continue;
            }
            if result.id == *value && use_counts.get(value) == Some(&1) {
                writes.insert(*value, *local);
            }
        }
    }
    writes
}

fn largest_window(function: &Function) -> usize {
    function
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .map(|instruction| match &instruction.operation {
            Operation::CallDirect { arguments, .. } | Operation::CallValue { arguments, .. } => {
                call_argument_width(arguments)
            }
            Operation::SpawnTask { arguments, .. }
            | Operation::SpawnDetachedTask { arguments, .. }
            | Operation::Intrinsic { arguments, .. } => arguments.len(),
            Operation::MakeClosure { captures, .. } => captures.len(),
            Operation::MakeRecord { fields, .. } | Operation::MakeEnum { fields, .. } => {
                fields.len()
            }
            Operation::MakeArray(values) => values.len(),
            Operation::MakeDictionary(pairs) => pairs.len().saturating_mul(2),
            Operation::StoreGlobalIndexPath { indexes, .. } => indexes.len().saturating_add(1),
            Operation::UpdateRecord { fields, .. } => fields.len().saturating_mul(2),
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

fn call_argument_width(arguments: &[ValueId]) -> usize {
    if arguments.len() == 1 {
        0
    } else {
        arguments.len()
    }
}

fn lowest_free(first_temporary: usize, used: &BTreeSet<u16>) -> Result<Register, CompileError> {
    for index in first_temporary..usize::from(Register::MAX.get()) + 1 {
        let register =
            Register::try_from_index(index).map_err(|error| limit_error(&error.to_string()))?;
        if !used.contains(&register.get()) {
            return Ok(register);
        }
    }
    Err(limit_error("function requires more addressable registers"))
}

fn last_uses(function: &Function) -> BTreeMap<ValueId, usize> {
    let mut uses = BTreeMap::new();
    let mut position = 0_usize;
    for block in &function.blocks {
        for instruction in &block.instructions {
            for value in operation_values(&instruction.operation) {
                uses.insert(value, position);
            }
            position = position.saturating_add(1);
        }
        if let Some(terminator) = block.terminators.first() {
            for value in terminator_values(terminator) {
                uses.insert(value, position);
            }
        }
        position = position.saturating_add(1);
    }
    uses
}

fn operation_values(operation: &Operation) -> Vec<ValueId> {
    match operation {
        Operation::Const(_)
        | Operation::ReadLocal(_)
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

fn terminator_values(terminator: &Terminator) -> Vec<ValueId> {
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
        Terminator::Return(value) => value.iter().copied().collect(),
        Terminator::Panic(value) => vec![*value],
    }
}

fn limit_error(message: &str) -> CompileError {
    internal_compiler_error(
        format!("Register allocation failed: {message}."),
        "Split the program into smaller functions or report this compiler invariant failure.",
        1,
        1,
    )
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests;
