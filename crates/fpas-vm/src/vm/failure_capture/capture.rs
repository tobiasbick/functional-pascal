//! Bounded pre-unwind frame and binding capture through the shared inspection core.

use super::{CaptureState, FailureFrame, FailureSnapshot, MAX_FRAMES, MAX_STRING};
use crate::vm::{
    inspection::{
        frames::{self, Boundary, Frame},
        values::{ValueLimits, budget::ValueBudget},
    },
    worker::Worker,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

/// Value bounds shared by original faults, peers, and globals.
pub(super) const LIMITS: ValueLimits = ValueLimits {
    max_children: 256,
    max_string_chars: MAX_STRING,
    max_summary_chars: MAX_STRING,
    max_depth: 16,
};

impl CaptureState {
    pub(super) fn capture(self: &Arc<Self>, worker: &Worker, snapshot: &mut FailureSnapshot) {
        self.capture_at(worker, snapshot, Boundary::Continuation);
    }

    /// Capture a live peer at its next instruction without executing it.
    pub(in crate::vm) fn capture_peer(self: &Arc<Self>, worker: &Worker) -> Arc<FailureSnapshot> {
        self.capture_observation(worker, Boundary::Stopped)
    }

    /// Retain this task's propagation stack without substituting the foreign cause's frames.
    pub(in crate::vm) fn capture_handling(
        self: &Arc<Self>,
        worker: &Worker,
    ) -> Arc<FailureSnapshot> {
        self.capture_observation(worker, Boundary::Fault)
    }

    fn capture_observation(
        self: &Arc<Self>,
        worker: &Worker,
        boundary: Boundary,
    ) -> Arc<FailureSnapshot> {
        let mut snapshot = FailureSnapshot {
            id: 0,
            task_id: Some(worker.task_id),
            entry_function: worker.capture_record.as_ref().map(|record| record.function),
            frames: Vec::new(),
            total_frames: None,
            reason: None,
            lease: super::Lease::default(),
        };
        self.capture_at(worker, &mut snapshot, boundary);
        if worker.capture_callback {
            snapshot.reason = snapshot.reason.or(Some("storage_unavailable"));
        }
        Arc::new(snapshot)
    }

    fn capture_at(
        self: &Arc<Self>,
        worker: &Worker,
        snapshot: &mut FailureSnapshot,
        boundary: Boundary,
    ) {
        let Ok(_access) = worker.domain.try_access() else {
            snapshot.reason = Some("storage_unavailable");
            return;
        };
        let (mut usage, mut budget) = self.budget();
        let before = (budget.values, budget.bytes);
        let offset = snapshot.frames.len();
        // Capture is diagnostic work. A failed capture must not replace the program's error
        // or leave partially retained data outside the shared budget.
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut captured = Vec::new();
            let mut omission = None;
            if offset == 0 && !budget.charge(std::mem::size_of::<FailureSnapshot>()) {
                return (captured, None, budget.exhausted);
            }
            let count = frames::visit(
                worker,
                boundary,
                MAX_FRAMES.saturating_sub(offset),
                |owner, frame, depth| {
                    if omission.is_some() {
                        return;
                    }
                    match capture_frame(owner, frame, offset + depth, &mut budget) {
                        Some(frame) => captured.push(frame),
                        None => omission = Some(budget.exhausted.unwrap_or("missing_metadata")),
                    }
                },
            );
            let reason = omission.or(count.unavailable_reason).or_else(|| {
                count
                    .total
                    .filter(|total| total.saturating_add(offset) > MAX_FRAMES)
                    .map(|_| "frame_limit")
            });
            (captured, count.total, reason)
        }));
        match result {
            Ok((frames, total, reason)) => {
                snapshot.lease.charge(self, &mut usage, before, &budget);
                snapshot.frames.extend(frames);
                snapshot.total_frames = if offset == 0 {
                    total
                } else {
                    snapshot
                        .total_frames
                        .zip(total)
                        .map(|(a, b)| a.saturating_add(b))
                };
                snapshot.reason = snapshot
                    .reason
                    .filter(|reason| *reason != "no_live_frame")
                    .or(reason);
            }
            Err(_) => snapshot.reason = Some("capture_failed"),
        }
    }
}

/// Captures a stopped or fault frame using the caller's shared value budget.
pub(in crate::vm) fn capture_frame(
    worker: &Worker,
    frame: Frame,
    depth: usize,
    budget: &mut ValueBudget,
) -> Option<FailureFrame> {
    crate::vm::inspection::detached_frame::capture(worker, frame, depth, budget, LIMITS, usize::MAX)
}
