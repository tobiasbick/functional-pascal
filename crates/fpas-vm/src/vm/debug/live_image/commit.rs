//! Prepared address mapping for one atomic live-image commit.

use std::sync::Arc;

use fpas_bytecode::{
    FunctionId, InstructionAddress, TraceRoots, VerifiedExecutable, classify_values,
};

use crate::vm::worker::Worker;

/// Candidate image plus total function-local instruction remapping.
pub(in crate::vm::debug) struct PreparedLiveImageCommit {
    candidate: Arc<VerifiedExecutable>,
    functions: Vec<FunctionAddressMap>,
}

#[derive(Clone, Copy)]
struct FunctionAddressMap {
    current_start: u32,
    current_end: u32,
    candidate_start: u32,
    candidate_end: u32,
}

impl PreparedLiveImageCommit {
    /// Prepare the candidate and all address ranges before mutable state changes.
    pub(in crate::vm::debug) fn new(
        current: &VerifiedExecutable,
        candidate: Arc<VerifiedExecutable>,
    ) -> Option<Self> {
        let functions = current
            .executable()
            .functions
            .iter()
            .zip(&candidate.executable().functions)
            .map(|(current, candidate)| FunctionAddressMap {
                current_start: current.code.start.get(),
                current_end: current.code.end.get(),
                candidate_start: candidate.code.start.get(),
                candidate_end: candidate.code.end.get(),
            })
            .collect::<Vec<_>>();
        if functions.len() != current.executable().functions.len()
            || functions.len() != candidate.executable().functions.len()
        {
            return None;
        }
        Some(Self {
            candidate,
            functions,
        })
    }

    /// Borrow the fully verified candidate owned by this transaction.
    pub(in crate::vm::debug) fn candidate(&self) -> &Arc<VerifiedExecutable> {
        &self.candidate
    }

    /// Verify that every instruction identity in a worker has a candidate address.
    pub(in crate::vm::debug) fn validates(&self, worker: &Worker) -> bool {
        self.retained_values_compatible(worker)
            && self.remap_index(worker.function, worker.ip).is_some()
            && self.remap_any_address(worker.current_address).is_some()
            && worker
                .call_stack
                .iter()
                .all(|frame| self.remap_index(frame.function, frame.ip).is_some())
            && worker.suppressed_initializers.iter().all(|target| {
                self.remap_address(target.function, target.instruction)
                    .is_some()
            })
    }

    /// Existing address mapping does not authorize retained callable targets. Until a
    /// graph can be rebound atomically, reject it before switching any worker/image.
    fn retained_values_compatible(&self, worker: &Worker) -> bool {
        let mut compatible = true;
        let mut inspect = |value: &fpas_bytecode::Value| {
            compatible &= classify_values(std::slice::from_ref(value)).is_ok_and(|graph| {
                graph
                    .executable_identities
                    .iter()
                    .all(|identity| identity == self.candidate.identity())
            });
        };
        if worker.trace_roots(&mut inspect).is_err() {
            return false;
        }
        if worker
            .domain
            .trace_registered_roots(&mut |_, value| inspect(value))
            .is_err()
        {
            return false;
        }
        let Ok(globals) = worker.globals.try_read() else {
            return false;
        };
        for value in globals.iter().flatten() {
            inspect(value);
        }
        compatible
    }

    /// Switch one prevalidated worker to the candidate image without allocation.
    pub(in crate::vm::debug) fn apply(&self, worker: &mut Worker) {
        worker.ip = self.remap_index_or_same(worker.function, worker.ip);
        worker.current_address = self
            .remap_any_address(worker.current_address)
            .unwrap_or(worker.current_address);
        for frame in &mut worker.call_stack {
            frame.ip = self.remap_index_or_same(frame.function, frame.ip);
        }
        for target in &mut worker.suppressed_initializers {
            target.instruction = self.remap_address_or_same(target.function, target.instruction);
        }
        worker.callback_worker.borrow_mut().take();
        worker.executable = Arc::clone(&self.candidate);
    }

    fn remap_index(&self, function: FunctionId, instruction: usize) -> Option<usize> {
        let instruction = u32::try_from(instruction).ok()?;
        usize::try_from(self.remap_raw(function, instruction)?).ok()
    }

    fn remap_address(
        &self,
        function: FunctionId,
        instruction: InstructionAddress,
    ) -> Option<InstructionAddress> {
        self.remap_raw(function, instruction.get())
            .map(InstructionAddress::new)
    }

    fn remap_any_address(&self, instruction: InstructionAddress) -> Option<InstructionAddress> {
        let (index, _) = self.functions.iter().enumerate().find(|(_, mapping)| {
            mapping.current_start <= instruction.get() && instruction.get() < mapping.current_end
        })?;
        let function = FunctionId::try_from_index(index).ok()?;
        self.remap_address(function, instruction)
    }

    fn remap_index_or_same(&self, function: FunctionId, instruction: usize) -> usize {
        self.remap_index(function, instruction)
            .unwrap_or(instruction)
    }

    fn remap_address_or_same(
        &self,
        function: FunctionId,
        instruction: InstructionAddress,
    ) -> InstructionAddress {
        self.remap_address(function, instruction)
            .unwrap_or(instruction)
    }

    fn remap_raw(&self, function: FunctionId, instruction: u32) -> Option<u32> {
        let mapping = self.functions.get(usize::from(function.get()))?;
        if instruction < mapping.current_start || instruction > mapping.current_end {
            return None;
        }
        let offset = instruction.checked_sub(mapping.current_start)?;
        let candidate = mapping.candidate_start.checked_add(offset)?;
        (candidate <= mapping.candidate_end).then_some(candidate)
    }
}
