//! Resource tickets preserve issue order without filling native slots with lock waiters.

use crate::{SourceLocation, stdlib::StdError};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Lane {
    issued: u64,
    head: u64,
    skipped: BTreeSet<u64>,
}

#[derive(Default)]
pub(in crate::execution::vm::hosted) struct Order(Mutex<[Lane; 3]>);

pub(super) struct Ticket {
    order: Arc<Order>,
    ids: [Option<u64>; 3],
}

impl Order {
    pub(super) fn ticket(
        self: &Arc<Self>,
        lanes: [bool; 3],
        location: SourceLocation,
    ) -> Result<Ticket, StdError> {
        let mut state = self.0.lock().unwrap_or_else(|p| p.into_inner());
        if state
            .iter()
            .zip(lanes)
            .any(|(lane, used)| used && lane.issued == u64::MAX)
        {
            return Err(crate::stdlib::error::std_runtime_error(
                fpas_diagnostics::codes::RUNTIME_HOST_IO_FAILURE,
                "Console command sequence exhausted",
                "Start a new VM instance.",
                location,
            ));
        }
        let ids = std::array::from_fn(|index| {
            if lanes[index] {
                let id = state[index].issued;
                state[index].issued += 1;
                Some(id)
            } else {
                None
            }
        });
        Ok(Ticket {
            order: self.clone(),
            ids,
        })
    }
}
impl Ticket {
    pub(super) fn ready(&self) -> bool {
        let state = self.order.0.lock().unwrap_or_else(|p| p.into_inner());
        self.ids
            .iter()
            .zip(state.iter())
            .all(|(id, lane)| id.is_none_or(|id| lane.head == id))
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        let mut state = self.order.0.lock().unwrap_or_else(|p| p.into_inner());
        for (id, lane) in self.ids.iter().zip(state.iter_mut()) {
            let Some(id) = *id else { continue };
            if id == lane.head {
                lane.head += 1;
                while lane.skipped.remove(&lane.head) {
                    lane.head += 1;
                }
            } else {
                lane.skipped.insert(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn cancelled_commands_leave_no_holes_and_resources_progress_independently() {
        let order = Arc::new(Order::default());
        let output = order.ticket([true, false, false], location()).unwrap();
        let cancelled = order.ticket([true, false, false], location()).unwrap();
        let next = order.ticket([true, false, false], location()).unwrap();
        let input = order.ticket([false, true, false], location()).unwrap();
        assert!(output.ready());
        assert!(input.ready());
        assert!(!next.ready());
        drop(cancelled);
        assert!(!next.ready());
        drop(output);
        assert!(next.ready());
        drop(next);
        drop(input);
        assert!(order.ticket([true; 3], location()).unwrap().ready());
    }

    #[test]
    fn terminal_handoffs_reserve_both_resources_in_one_order() {
        let order = Arc::new(Order::default());
        let key = order.ticket([false, false, true], location()).unwrap();
        let handoff = order.ticket([true, false, true], location()).unwrap();
        let output = order.ticket([true, false, false], location()).unwrap();
        assert!(!handoff.ready());
        assert!(!output.ready());
        drop(key);
        assert!(handoff.ready());
        assert!(!output.ready());
        drop(handoff);
        assert!(output.ready());
    }

    #[test]
    fn exhausted_sequences_do_not_partially_reserve_other_resources() {
        let order = Arc::new(Order::default());
        {
            let mut state = order.0.lock().unwrap();
            state[2].issued = u64::MAX - 1;
            state[2].head = u64::MAX - 1;
        }
        let last = order.ticket([false, false, true], location()).unwrap();
        assert!(last.ready());
        assert!(order.ticket([true, false, true], location()).is_err());
        assert!(
            order
                .ticket([true, false, false], location())
                .unwrap()
                .ready()
        );
        drop(last);
    }
}
