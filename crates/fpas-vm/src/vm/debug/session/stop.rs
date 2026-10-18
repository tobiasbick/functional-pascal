//! Stops at the failing instruction's stored coordinates or the next sequence point.

use super::*;
use crate::vm::debug::types::SourceLocation;

pub(super) fn stop_at_worker(
    executable: &VerifiedExecutable,
    worker: &Worker,
    reason: DebugStopReason,
    breakpoint_ids: Vec<u64>,
    diagnostic: Option<crate::vm::VmError>,
) -> DebugStop {
    let instruction = if reason == DebugStopReason::RuntimeError {
        worker.current_address
    } else {
        InstructionAddress::try_from_index(worker.ip).unwrap_or(worker.current_address)
    };
    let point = crate::vm::debug::breakpoints::point_at(executable, worker.function, instruction);
    let fault_location = if reason == DebugStopReason::RuntimeError {
        diagnostic
            .as_ref()
            .and_then(|error| error.span)
            .and_then(|span| {
                // Runtime spans can retain valid coordinates without source byte offsets.
                let image = executable.executable();
                let source = image
                    .source_map
                    .sources
                    .get(span.source_id() as usize)
                    .and_then(|name| image.strings.get(*name))?;
                Some(SourceLocation {
                    source: source.into(),
                    line: span.line(),
                    column: span.column(),
                })
            })
    } else {
        None
    };
    DebugStop {
        reason,
        task_id: worker.task_id,
        location: fault_location
            .or_else(|| {
                point.and_then(|point| {
                    crate::vm::debug::breakpoints::source_location(executable, point)
                })
            })
            .or_else(|| {
                if reason != DebugStopReason::RuntimeError {
                    return None;
                }
                let image = executable.executable();
                let position = crate::vm::inspection::location::position_at(
                    image,
                    worker.function,
                    instruction,
                )?;
                Some(SourceLocation {
                    source: crate::vm::inspection::location::source_path(image, position)?.into(),
                    line: position.line,
                    column: position.column,
                })
            }),
        instruction: instruction.get(),
        call_depth: worker.call_stack.len(),
        breakpoint_id: breakpoint_ids.first().copied(),
        breakpoint_ids,
        diagnostic,
    }
}
