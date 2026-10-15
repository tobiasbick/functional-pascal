//! Bounded frame visitation at stopped and failing instruction boundaries.

use crate::vm::worker::Worker;
use fpas_bytecode::{FunctionId, InstructionAddress};

/// Which active instruction is being inspected.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::vm) enum Boundary {
    /// The next instruction, before execution at a debugger stop.
    Stopped,
    /// The last dispatched instruction, before failure unwind.
    Fault,
    /// A fault or hosted caller whose nested callback has already been captured.
    Continuation,
}

/// Register window and source addresses for one logical frame.
#[derive(Clone, Copy)]
pub(in crate::vm) struct Frame {
    /// Function owning this frame's code and metadata.
    pub function: FunctionId,
    /// Active instruction or saved call site.
    pub instruction: InstructionAddress,
    /// Resume address used to decide whether an initializer is still pending.
    pub next_instruction: InstructionAddress,
    /// First register in this frame's window.
    pub base: usize,
}

/// Visit at most `maximum` frames, returning the total number of known frames.
/// Borrowed callback workers never escape the visitor and no frame list is cloned.
pub(in crate::vm) fn visit(
    worker: &Worker,
    boundary: Boundary,
    maximum: usize,
    mut visitor: impl FnMut(&Worker, Frame, usize),
) -> FrameCount {
    fn walk(
        worker: &Worker,
        boundary: Boundary,
        maximum: usize,
        depth: &mut usize,
        callbacks: usize,
        visitor: &mut impl FnMut(&Worker, Frame, usize),
    ) -> Result<(), &'static str> {
        if boundary == Boundary::Fault && worker.callback_failed.get() {
            if callbacks >= 256 {
                return Err("frame_limit");
            }
            let callback = worker.callback_worker.try_borrow().map_err(|_| "busy")?;
            let callback = callback.as_deref().ok_or("storage_unavailable")?;
            walk(callback, boundary, maximum, depth, callbacks + 1, visitor)?;
        }
        let next = InstructionAddress::try_from_index(worker.ip).unwrap_or(worker.current_address);
        let instruction = if boundary != Boundary::Stopped {
            // An inline hosted callback may have returned before its continuation fails.
            // Its Return address is no longer an instruction in the restored caller.
            let caller_site = InstructionAddress::try_from_index(worker.ip.saturating_sub(1))
                .unwrap_or(worker.current_address);
            let code =
                &worker.executable.executable().functions[usize::from(worker.function.get())].code;
            if !code.contains(worker.current_address) && code.contains(caller_site) {
                caller_site
            } else {
                worker.current_address
            }
        } else {
            next
        };
        if *depth < maximum {
            visitor(
                worker,
                Frame {
                    function: worker.function,
                    instruction,
                    next_instruction: next,
                    base: worker.base,
                },
                *depth,
            );
        }
        *depth = depth.saturating_add(1);
        for frame in worker.call_stack.iter().rev() {
            if *depth < maximum {
                visitor(
                    worker,
                    Frame {
                        function: frame.function,
                        instruction: InstructionAddress::try_from_index(frame.ip.saturating_sub(1))
                            .unwrap_or(worker.current_address),
                        next_instruction: InstructionAddress::try_from_index(frame.ip)
                            .unwrap_or(worker.current_address),
                        base: frame.base,
                    },
                    *depth,
                );
            }
            *depth = depth.saturating_add(1);
        }
        Ok(())
    }
    let mut depth = 0;
    let result = walk(worker, boundary, maximum, &mut depth, 0, &mut visitor);
    FrameCount {
        total: result.as_ref().ok().map(|_| depth),
        unavailable_reason: result.err(),
    }
}

/// Whether every frame owner could be observed without blocking.
pub(in crate::vm) struct FrameCount {
    /// Full known frame count, including omitted frames; unknown when an owner is busy.
    pub total: Option<usize>,
    /// Stable reason a callback frame chain could not be counted or inspected.
    pub unavailable_reason: Option<&'static str>,
}
