//! Deterministic lowest-free-register linear-scan allocation.

mod aliases;
mod dead_values;
mod loop_liveness;
mod uses;

use std::collections::{BTreeMap, BTreeSet};

use fpas_bytecode::Register;
use fpas_ir::{BlockId, Function, LocalId, Operation, ValueId};

use crate::CompileError;
use crate::error::internal_compiler_error;

use self::uses::{operation_values, terminator_values};

pub(super) struct Allocation {
    locals: BTreeMap<LocalId, Register>,
    values: BTreeMap<ValueId, Register>,
    // Temporary values paired with the instruction that reads them for the last time.
    final_reads: BTreeSet<(BlockId, usize, ValueId)>,
    // Constants that no emitted instruction reads; they get no register and no code.
    dead_constants: BTreeSet<ValueId>,
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
        let aliased_reads = aliases::aliased_local_reads(function, &coalesced_writes);
        let dead_constants = dead_values::dead_constants(function, &last_uses);
        let mut active: Vec<(ValueId, usize, Register)> = Vec::new();
        let mut instruction_sites = BTreeMap::new();
        let mut temporaries = Vec::new();
        let mut occupied = BTreeSet::new();
        let mut position = 0_usize;
        let mut high_water = next_fixed;
        for block in &function.blocks {
            if !block.parameters.is_empty() {
                return Err(limit_error(
                    "register allocation received unsupported block parameters",
                ));
            }
            for (index, instruction) in block.instructions.iter().enumerate() {
                instruction_sites.insert(position, (block.id, index));
                active.retain(|(_, last_use, register)| {
                    if *last_use < position {
                        occupied.remove(&register.get());
                        false
                    } else {
                        true
                    }
                });
                if let Some(result) = instruction
                    .result
                    .filter(|result| !dead_constants.contains(&result.id))
                {
                    let local = coalesced_writes
                        .get(&result.id)
                        .or_else(|| aliased_reads.get(&result.id));
                    let register = if let Some(local) = local {
                        locals.get(local).copied().ok_or_else(|| {
                            limit_error(&format!(
                                "coalesced local {} has no allocated register",
                                local.get()
                            ))
                        })?
                    } else {
                        let register = lowest_free(next_fixed, &occupied)?;
                        high_water = high_water.max(usize::from(register.get()) + 1);
                        let last_use = last_uses.get(&result.id).copied().unwrap_or(position);
                        occupied.insert(register.get());
                        active.push((result.id, last_use, register));
                        temporaries.push(result.id);
                        register
                    };
                    values.insert(result.id, register);
                }
                position = position.saturating_add(1);
            }
            position = position.saturating_add(1);
        }
        let final_reads = temporaries
            .into_iter()
            .filter_map(|value| {
                let (block, index) = instruction_sites.get(last_uses.get(&value)?)?;
                Some((*block, *index, value))
            })
            .collect();
        let call_window = Register::try_from_index(high_water)
            .map_err(|error| limit_error(&error.to_string()))?;
        let window_size = largest_window(function);
        let register_count = u16::try_from(high_water.saturating_add(window_size))
            .map_err(|_| limit_error("register count exceeds the portable u16 frame limit"))?;
        Ok(Self {
            locals,
            values,
            final_reads,
            dead_constants,
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

    /// Whether `value` is a constant that no emitted instruction reads.
    pub fn is_dead_constant(&self, value: ValueId) -> bool {
        self.dead_constants.contains(&value)
    }

    /// Whether instruction `index` of `block` is the last reader of temporary `value`.
    ///
    /// The allocator reuses the register after this read, so the instruction may consume the
    /// value. Parameters, locals, and values stored directly into locals never qualify.
    pub fn is_final_read(&self, value: ValueId, block: BlockId, index: usize) -> bool {
        self.final_reads.contains(&(block, index, value))
    }

    /// Registers of temporaries that `instruction` (at `index` in `block`) reads exactly once and
    /// for the last time, so a move out of them may transfer the value instead of cloning it.
    pub fn consumable_registers(
        &self,
        block: BlockId,
        index: usize,
        instruction: &fpas_ir::Instruction,
    ) -> Vec<u16> {
        let operands = operation_values(&instruction.operation);
        operands
            .iter()
            .filter(|value| operands.iter().filter(|other| other == value).count() == 1)
            .filter(|value| self.is_final_read(**value, block, index))
            .filter_map(|value| self.values.get(value).map(|register| register.get()))
            .collect()
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

/// Last register read of each value; operands encoded as immediates do not read a register.
fn last_uses(function: &Function) -> BTreeMap<ValueId, usize> {
    let mut uses = BTreeMap::new();
    let mut position = 0_usize;
    for block in &function.blocks {
        for (index, instruction) in block.instructions.iter().enumerate() {
            let immediate = dead_values::immediate_operand(block, index);
            for value in operation_values(&instruction.operation) {
                if Some(value) != immediate {
                    uses.insert(value, position);
                }
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
    loop_liveness::extend_across_back_edges(function, &mut uses);
    uses
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
