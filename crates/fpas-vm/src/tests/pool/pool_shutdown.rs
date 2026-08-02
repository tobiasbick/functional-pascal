use crate::vm::{CallFrame, GraphState, SharedState, TaskTimers, Worker};
use fpas_bytecode::{Chunk, Op, Value};
use fpas_std::{Console, KeyInput, TextInput};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};

use crate::tests::helpers::{emit_constant, loc};

#[test]
fn pool_tasks_stop_without_side_effects_after_abort_flag() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str(("late".to_string()).into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let shared = Arc::new(SharedState {
        chunk: Arc::new(chunk),
        program_args: Vec::new(),
        globals: RwLock::new(HashMap::new()),
        task_queue: Mutex::new(VecDeque::new()),
        task_available: Condvar::new(),
        task_timers: TaskTimers::new(),
        accept_task_timers: AtomicBool::new(true),
        task_results: Mutex::new(HashMap::new()),
        task_completions: Mutex::new(HashSet::new()),
        task_results_available: Condvar::new(),
        completed_task_count: AtomicU64::new(0),
        next_task_id: AtomicU64::new(1),
        console: Mutex::new(Console::new()),
        text_input: Mutex::new(TextInput::new()),
        key_input: Mutex::new(KeyInput::new()),
        graph: Mutex::new(GraphState::default()),
        shutdown: AtomicBool::new(true),
        abort_spawned_bytecode: AtomicBool::new(true),
    });

    let mut worker = Worker::new_pool(Arc::clone(&shared));
    worker.load_task(crate::vm::TaskState {
        id: 1,
        ip: 0,
        stack: Vec::new(),
        call_stack: Vec::<CallFrame>::new(),
        retain_result: false,
    });

    worker
        .run()
        .expect("abort flag should stop pool tasks cleanly");

    let output = shared
        .console
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .output()
        .clone();
    assert!(
        output.lines.is_empty(),
        "pool task should not emit output after cooperative abort"
    );
    assert!(shared.shutdown.load(Ordering::Acquire));
}
