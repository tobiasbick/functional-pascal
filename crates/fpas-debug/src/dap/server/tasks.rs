//! Stable runtime-task to positive DAP-thread identity mapping and live cache.

use std::collections::{BTreeMap, HashMap};

use serde_json::{Value, json};

struct ThreadEntry {
    thread_id: u64,
    name: String,
    active: bool,
}

/// Session-local mapping between runtime task IDs and positive DAP thread IDs.
pub(super) struct ThreadMap {
    tasks: BTreeMap<u64, ThreadEntry>,
    thread_to_task: HashMap<u64, u64>,
    next_thread_id: u64,
}

impl ThreadMap {
    /// Create a map with runtime main task `0` fixed to DAP thread `1`.
    pub(super) fn new() -> Self {
        Self {
            tasks: BTreeMap::from([(
                0,
                ThreadEntry {
                    thread_id: 1,
                    name: "FPAS main".to_string(),
                    active: true,
                },
            )]),
            thread_to_task: HashMap::from([(1, 0)]),
            next_thread_id: 2,
        }
    }

    /// Return or allocate the stable DAP thread ID for one runtime task.
    pub(super) fn thread_id(&mut self, task_id: u64) -> u64 {
        if let Some(entry) = self.tasks.get_mut(&task_id) {
            entry.active = true;
            return entry.thread_id;
        }
        let thread_id = self.next_thread_id;
        self.next_thread_id = self.next_thread_id.saturating_add(1);
        self.tasks.insert(
            task_id,
            ThreadEntry {
                thread_id,
                name: format!("FPAS task {task_id}"),
                active: true,
            },
        );
        self.thread_to_task.insert(thread_id, task_id);
        thread_id
    }

    /// Refresh the active cache from one stopped JSONL task catalog.
    pub(super) fn synchronize(&mut self, tasks: &[Value]) {
        for entry in self.tasks.values_mut() {
            entry.active = false;
        }
        for task in tasks {
            if matches!(
                task.get("state").and_then(Value::as_str),
                Some("completed" | "cancelled")
            ) {
                continue;
            }
            let task_id = task.get("task_id").and_then(Value::as_u64).unwrap_or(0);
            let thread_id = self.thread_id(task_id);
            if let Some(entry) = self.tasks.get_mut(&task_id) {
                entry.thread_id = thread_id;
                entry.name = task
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("FPAS task")
                    .to_string();
            }
        }
    }

    /// Mark a task inactive while preserving its identity against reuse.
    pub(super) fn mark_exited(&mut self, task_id: u64) {
        if let Some(entry) = self.tasks.get_mut(&task_id) {
            entry.active = false;
        }
    }

    /// Return the last stable active DAP thread catalog in runtime task order.
    pub(super) fn active_threads(&self) -> Value {
        json!({
            "threads": self.tasks.values().filter(|entry| entry.active).map(|entry| {
                json!({"id":entry.thread_id,"name":entry.name})
            }).collect::<Vec<_>>()
        })
    }

    /// Resolve one previously allocated DAP thread ID.
    pub(super) fn task_id(&self, thread_id: u64) -> Option<u64> {
        self.thread_to_task.get(&thread_id).copied()
    }
}
