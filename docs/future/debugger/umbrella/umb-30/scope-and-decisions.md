# UMB-30 scope and decisions

## Shared invariants

1. The selected task is the task that caused the all-stop event. Operations on
   peer, waiting, sleeping, or externally blocked tasks remain in `UMB-40`.
2. Every command follows prepare, validate, then one infallible commit. A
   rejection changes no worker, scheduler result, waiter, stop generation, or
   adapter state.
3. Result expressions use the existing portable debugger evaluator and exact
   function result type before any lifecycle mutation.
4. A successful command executes no bytecode, scheduler turn, callback,
   cleanup handler, or host I/O. Normal execution resumes only after the
   command has committed and the client explicitly continues where required.
5. JSONL and DAP call the same session operation. VS Code maps to that adapter
   behavior and must not reconstruct VM state itself.
6. No FPAS syntax, semantics, or language documentation changes are in scope.

## `UMB-30A` — root and task entry completion

- Extend the existing selected-frame return operation instead of creating a
  competing return command.
- Returning a child task entry frame publishes exactly one typed success
  result, wakes eligible retained-handle waiters through scheduler state, and
  emits exactly one task-exit transition. Readiness may be refreshed, but no
  newly ready task may execute inside the command.
- Returning the root entry frame terminates the root with the supplied result
  and deterministically cancels remaining task/timer state. The command does
  not run user cleanup code.
- Entry completion is rejected for an ineligible task state, stale stop/frame,
  type mismatch, or result that cannot cross the task boundary. Every rejection
  is atomic.

## `UMB-30B` — runtime-error recovery

- A runtime failure is initially reported and retained exactly as today.
  Recovery is an explicit debugger action; it never silently retries the failed
  instruction or hides the original diagnostic event.
- Inner-frame recovery may replace the failed frame through an accepted forced
  return only if the scheduler can compare and transition the exact unconsumed
  failure back to runnable state atomically.
- Entry-frame recovery may replace the exact unconsumed retained failure with a
  typed success result and apply the `UMB-30A` completion rules.
- If a waiter could have consumed the failure, the stop is stale, or the
  scheduler cannot prove the exact failure identity, recovery is rejected
  without mutation.

## Completed-return replacement boundary

- An unconsumed retained task result has a stable task identity and may be
  replaceable if the scheduler can compare the old result and atomically
  publish a type-correct replacement.
- An ordinary callee return currently has no retained frame/completion identity
  after its frame is removed and destination register is written. Do not claim
  general completed-frame replacement without first adding bounded history and
  identity under `UMB-40D` or `UMB-80`.
- `U30-30` must implement the provable retained-result subset or record precise
  evidence and dependency. It must not rename a narrower feature as general
  completed-return replacement.

## `UMB-30C` — frame restart

- Restart reconstructs the selected live frame using its current parameter and
  capture values. Capture values preserve their exact cell handles.
- All local and temporary registers in that frame are cleared or initialized
  according to verified function metadata; the instruction pointer moves to
  the function entry; younger frames are discarded as one commit.
- Restarting a frame may cause already-observed side effects to occur again
  after a later continue. This is documented behavior, not rollback.
- Root/task entry restart follows the same reconstruction rule. Peer, waiting,
  sleeping, failed, and completed tasks remain outside this operation except
  for the explicit recovery path above.

## Source-initializer suppression boundary

- Suppression must use compiler-retained binding identity and initializer-store
  metadata that survives bytecode, unit, linker, and bundle round trips.
- Source name, line proximity, destination register alone, and heuristic
  instruction scanning are insufficient identities.
- If current metadata cannot prove the exact initializer store and binding
  lifetime, `U30-41` records the missing invariant and dependency. It must not
  implement a best-effort suppression mode.

## `UMB-30D` — instruction change feasibility

- An accepted destination must preserve function/frame identity, exception and
  cleanup state, register initialization, operand types, lexical scope, and a
  valid source mapping.
- The verifier proves the original control-flow graph, not arbitrary debugger
  jumps. `U30-50` first determines whether existing bytecode dataflow can prove
  safe destinations without a second execution engine.
- If only a bounded mode such as verified same-function sequence points is
  sound, record it explicitly as a narrower accepted operation. Unrestricted
  instruction changes remain rejected or dependent; they are not silently
  declared complete.

## Bounds

- No unbounded snapshots, completion history, source scans, or evaluation.
- Existing debugger expression and protocol limits remain authoritative.
- Any new metadata is portable across bytecode units, linked programs, and
  packaged bundles and has malformed-input tests.
