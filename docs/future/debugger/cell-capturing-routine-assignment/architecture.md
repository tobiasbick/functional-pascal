# Architecture

## Existing foundations

- Semantic capture analysis distinguishes `Value`, `Cell`, and
  `EnclosingCell`; compiler debug metadata now preserves exact declaration,
  owner-function, owner-binding, type, and capture-kind identities.
- `.fpascu`, linker, `.fpascp`, and bytecode verification preserve and validate
  lexical-owner and capture-source metadata.
- A runtime mutable capture is `Value::Cell(Arc<Mutex<Value>>)`. Cloning that
  value preserves cell identity, and VM closure creation marks a function
  `task_bound` when any capture is a cell or a nested task-bound function.
- `Worker` already has a runtime `task_id`, and debugger frame IDs already map
  to one stopped task and inspection generation.
- The existing mutation pipeline resolves a typed target, prepares a complete
  replacement, validates it, commits one root, refreshes stopped snapshots, and
  expires old handles only on success.

The missing foundations are runtime owner identity for task-bound functions, a
capture-source read that may return exact cell handles, and a destination policy
that prevents generic function assignment from widening this bounded slice.

## Runtime callable ownership

`SharedFunction` gains runtime-only ownership for task-bound values:

```text
SharedFunction
  function: FunctionId
  name: String
  captures: Vec<Value>
  task_bound: bool
  owner_task: Option<u64>
```

Required invariants:

- `task_bound = false` implies `owner_task = None`.
- A VM-created or debugger-created function with cell reachability has
  `task_bound = true` and `owner_task = Some(current_worker.task_id)`.
- Ordinary calls compare `owner_task` with the executing worker before frame
  entry. A mismatch is a stable runtime diagnostic.
- Task spawning retains its existing unconditional rejection of task-bound
  callables.
- The owner token is never serialized into program or unit artifacts and is
  never inferred from a display string.

This runtime check closes the gap created when debugger mutation changes a
function binding after static task-bound analysis has finished.

## Capture preparation flow

```text
setVariable / setExpression replacement
  -> resolve one mutable function target and its frame/task ownership
  -> resolve one unique executable named nested routine
  -> require verified lexical-owner and complete capture-source metadata
  -> require request frame == target frame == lexical-owner frame
  -> for each capture in ABI order:
       Value          -> clone and run existing bounded safe-graph validation
       Cell           -> require exact owner binding, cell-backed, Value::Cell
                         then Arc-clone the existing handle
       EnclosingCell  -> require exact owner capture binding, cell-backed,
                         Value::Cell, then Arc-clone the same handle
  -> prove portable routine signature against destination DebugType::Function
  -> construct SharedFunction(task_bound = true, owner_task = selected task)
  -> validate the completed runtime value without traversing cell payloads
  -> commit the existing frame-register root once
  -> refresh all stopped snapshots and expire old references once
```

The capture-source reader must return runtime `Value`s, not rendered retained
values. It checks binding ID, portable type, visibility, initialization, and
recorded representation before returning. It does not lock a `Cell` because
the identity itself, not a payload snapshot, is the capture.

## Intended ownership layout

```text
crates/fpas-bytecode/src/value/
  function.rs             — MODIFY: runtime task owner and constructor invariant
  mod.rs                  — MODIFY: explicit owned/non-owned function factories
  equal.rs                — MODIFY: include runtime ownership in value identity

crates/fpas-vm/src/vm/
  calls.rs                — MODIFY: stamp owner on closure creation; reject foreign call
  tasks/mod.rs            — CONFIRM: existing task-bound spawn rejection

crates/fpas-vm/src/vm/debug/inspection/
  capture_sources.rs      — MODIFY: exact Value/Cell/EnclosingCell source reads

crates/fpas-vm/src/vm/debug/mutation/function_value/
  captures.rs             — MODIFY/SPLIT: retain non-task-bound copy policy
  routine/captures.rs     — MODIFY: task-owned named-routine capture construction
  routine/task_owner.rs   — NEW if needed: focused owner and destination policy

crates/fpas-vm/src/vm/debug/session/
  mutation/replacement.rs — MODIFY: pass resolved frame/task target context

crates/fpas-vm/src/vm/debug/tests/
  cell_capturing_routine_assignment/ — NEW: focused identity, ownership, and atomicity

crates/fpas-debug/tests/
  cell_capturing_routine_assignment.rs      — NEW: JSONL contract
  dap_cell_capturing_routine_assignment.rs  — NEW: DAP parity/invalidation

editors/vscode/test/debugger_host/
  cell_capturing_routine_assignment.ts      — NEW: real Extension Host coverage

tests/debugger/fixtures/
  cell_capturing_routine_assignment.fpas    — NEW: Cell and EnclosingCell program
```

Recheck exact paths and line counts before implementation. Split a file that is
already near 400 lines rather than adding another concern to it.

## Atomicity and limits

- Resolve target and all source identities against one unchanged stopped
  generation.
- Charge immutable capture graphs to the existing depth and detached-value
  limits. Count cell handles but do not traverse or lock their payloads.
- Clone `Arc` handles only after all metadata needed to address them is proven;
  cloning is not a live-state mutation.
- Construct and validate the complete function value off to the side.
- Commit through the existing single-root mutation path. Success changes one
  destination and one inspection generation; failure changes neither.
- Poisoned, busy, or cyclic cell contents do not affect construction because
  construction never reads the payload. Normal invocation retains existing VM
  cell-access behavior.

## Protocol model

No protocol shape changes. JSONL and DAP use their standard assignment requests,
and VS Code continues to forward Variables/Watch edits. Successful assignment
returns the canonical function summary and invalidates variables once when DAP
invalidation was negotiated. Failure returns the shared machine error and hint,
emits no invalidation, and preserves old handles.
