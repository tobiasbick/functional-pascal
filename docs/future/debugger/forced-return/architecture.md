# Architecture

## Existing runtime boundary

`Worker` stores the active function, instruction pointer, register base, and a
`call_stack` of saved `CallFrame` values. Each saved frame already contains the
caller function, post-call instruction pointer, caller base, and optional
absolute return destination. Normal `return_from_call` pops one saved frame,
releases the callee register window, restores the caller, and stores the result.

Inspection assigns depth zero to the active frame and higher depths to saved
callers. Stop-local frame IDs already encode the snapshot generation, and the
session maps each frame back to its task. The missing pieces are a preflighted
debug-only transition, portable result-type metadata, and shared protocol
mapping.

## Metadata path

Add `result_type: Option<DebugTypeId>` to executable `FunctionDebugInfo` and the
equivalent object-local field to `ObjectFunctionDebugInfo`.

```text
IR function signature result TypeId
  -> compiler bytecode debug metadata
  -> relocatable object local debug type ID
  -> .fpascu serialization and validation
  -> linker debug-type reachability and ID translation
  -> executable FunctionDebugInfo
  -> .fpascp serialization and verification
  -> stopped-session forced-return validation
```

The verifier must reject out-of-range IDs. The linker must retain and translate
types referenced only by function results. `None` means unavailable metadata,
not Dynamic and not `unit`.

## Prepare and commit flow

```text
frame.return / fpas/forceReturn / VS Code command
  -> require stopped, non-runtime-failure session
  -> resolve current-generation frame and task
  -> require depth zero and task == last_stop.task_id
  -> require active worker with one saved caller
  -> read ReturnConvention and portable result_type
  -> procedure: require no expression, prepare Value::Unit
  -> function: require expression, evaluate once in active callee snapshot
  -> validate value category and declared portable result type
  -> preflight saved caller and optional destination without mutation
  -> pop exactly one saved caller through the dedicated debugger commit
  -> refresh last_stop and all inspection generations at restored caller
  -> return fresh caller frame and rendered result
```

The detached evaluator may execute only calls already allowed by current debug
effect policy. Evaluation completes before register release. The final runtime
transition performs no expression parsing, evaluation, allocation policy, or
protocol formatting.

## Atomicity boundary

Preflight checks all fallible structural conditions before the live frame is
popped:

- the active function and saved caller exist;
- the result metadata resolves inside the verified executable;
- the result value passes bounded type validation;
- the saved destination, when present, belongs to the caller register window;
- release bounds and restored instruction address are valid; and
- the selected worker is not suspended or failed.

After preflight, commit may only release the known callee window, restore the
saved frame, and store the already validated value. If this cannot be made
effectively infallible, implementation stops rather than exposing a partially
unwound stack.

## Ownership boundaries

```text
crates/fpas-bytecode/src/debug/callable.rs
  — portable optional function result type

crates/fpas-compiler/src/bytecode/debug.rs
  — populate result metadata from the IR signature

crates/fpas-unit/src/object/ and crates/fpas-linker/src/
  — serialize, validate, retain, translate, and link result type IDs

crates/fpas-vm/src/vm/debug/forced_return/
  — eligibility, value validation, preflight, and one-frame commit

crates/fpas-vm/src/vm/debug/session/forced_return.rs
  — stopped-state, selected-task, evaluation, refresh, and result orchestration

crates/fpas-debug/src/jsonl/server/forced_return.rs
  — machine-friendly JSONL request and response mapping

crates/fpas-debug/src/dap/server/forced_return.rs
  — custom DAP mapping and negotiated invalidation

editors/vscode/src/debugger/forcedReturnCommand.ts
  — active-frame command, prompt, request, and refresh UX only
```

No adapter owns return semantics. No language parser, semantic rule, or source
runtime return behavior changes.
