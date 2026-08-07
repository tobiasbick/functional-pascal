//! Deterministic lowest-free-register linear-scan allocation.

use std::collections::{BTreeMap, BTreeSet};

use fpas_bytecode::Register;
use fpas_ir::{Function, LocalId, Operation, Terminator, ValueId};

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) struct Allocation {
    locals: BTreeMap<LocalId, Register>,
    values: BTreeMap<ValueId, Register>,
    pub register_count: u16,
}

impl Allocation {
    pub fn build(function: &Function) -> Result<Self, CompileError> {
        if !function.parameters.is_empty() || !function.captures.is_empty() {
            return Err(limit_error(
                "P3 register allocation does not accept parameters or captures",
            ));
        }
        let mut locals = BTreeMap::new();
        for (index, local) in function.locals.iter().enumerate() {
            let register =
                Register::try_from_index(index).map_err(|error| limit_error(&error.to_string()))?;
            locals.insert(local.id, register);
        }

        let last_uses = last_uses(function);
        let mut values = BTreeMap::new();
        let mut active: Vec<(ValueId, usize, Register)> = Vec::new();
        let mut position = 0_usize;
        let mut high_water = function.locals.len();
        for block in &function.blocks {
            if !block.parameters.is_empty() {
                return Err(limit_error(
                    "P3 register allocation does not accept block parameters",
                ));
            }
            for instruction in &block.instructions {
                active.retain(|(_, last_use, _)| *last_use >= position);
                if let Some(result) = instruction.result {
                    let used: BTreeSet<u16> = active
                        .iter()
                        .map(|(_, _, register)| register.get())
                        .chain(locals.values().map(|register| register.get()))
                        .collect();
                    let register = lowest_free(function.locals.len(), &used)?;
                    high_water = high_water.max(usize::from(register.get()) + 1);
                    let last_use = last_uses.get(&result.id).copied().unwrap_or(position);
                    active.push((result.id, last_use, register));
                    values.insert(result.id, register);
                }
                position = position.saturating_add(1);
            }
            position = position.saturating_add(1);
        }
        let register_count = u16::try_from(high_water)
            .map_err(|_| limit_error("register count exceeds the portable u16 frame limit"))?;
        Ok(Self {
            locals,
            values,
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
        | Operation::Yield => Vec::new(),
        Operation::WriteLocal { value, .. }
        | Operation::StoreGlobal { value, .. }
        | Operation::CellRead(value) => vec![*value],
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
        Operation::StoreField { record, value, .. }
        | Operation::CellWrite {
            cell: record,
            value,
        } => vec![*record, *value],
        Operation::TestVariant { value, .. } => vec![*value],
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
