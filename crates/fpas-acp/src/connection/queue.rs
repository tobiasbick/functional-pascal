//! Bounded admission that counts entries already borrowed by a transport consumer.

use super::Fault;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

struct State<T> {
    items: VecDeque<Item<T>>,
    count: usize,
    bytes: usize,
    closed: bool,
}

pub(crate) struct Queue<T> {
    state: Mutex<State<T>>,
    entries: usize,
    maximum: usize,
    pub changed: Notify,
}

pub(crate) struct Item<T> {
    pub value: T,
    _credit: Credit<T>,
}

struct Credit<T> {
    queue: std::sync::Weak<Queue<T>>,
    bytes: usize,
}

impl<T> Item<T> {
    pub fn into_value(self) -> T {
        self.value
    }

    pub fn split(self) -> (T, impl Send + 'static)
    where
        T: Send + 'static,
    {
        (self.value, self._credit)
    }
}

impl<T> Drop for Credit<T> {
    fn drop(&mut self) {
        if let Some(queue) = self.queue.upgrade() {
            let mut state = queue
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state.count -= 1;
            state.bytes -= self.bytes;
            drop(state);
            queue.changed.notify_waiters();
        }
    }
}

impl<T> Queue<T> {
    pub fn maximum(&self) -> usize {
        self.maximum
    }
    pub fn new(entries: usize, maximum: usize) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                items: VecDeque::new(),
                count: 0,
                bytes: 0,
                closed: false,
            }),
            entries,
            maximum,
            changed: Notify::new(),
        })
    }

    pub fn push(self: &Arc<Self>, value: T, bytes: usize) -> Result<bool, Fault> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.closed {
            return Err(Fault::Closed);
        }
        if state.count == self.entries || bytes > self.maximum.saturating_sub(state.bytes) {
            return Ok(false);
        }
        state.count += 1;
        state.bytes += bytes;
        state.items.push_back(Item {
            value,
            _credit: Credit {
                queue: Arc::downgrade(self),
                bytes,
            },
        });
        drop(state);
        self.changed.notify_waiters();
        Ok(true)
    }

    pub fn pop(&self) -> Option<Item<T>> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .items
            .pop_front()
    }

    pub fn close(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let first = !state.closed;
        state.closed = true;
        drop(state);
        self.changed.notify_waiters();
        first
    }

    pub fn closed(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .closed
    }

    pub fn discard(&self) {
        let items = std::mem::take(
            &mut self
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .items,
        );
        drop(items);
    }

    pub async fn receive(&self) -> Option<Item<T>> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(item) = self.pop() {
                return Some(item);
            }
            if self.closed() {
                return None;
            }
            notified.await;
        }
    }
}
