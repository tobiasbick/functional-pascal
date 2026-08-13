# Scope and decisions

## Accepted scope

- FRT-S01: expose one protocol-neutral forced-return operation through JSONL
  `frame.return`, DAP `fpas/forceReturn`, and the VS Code command
  `functionalPascal.debug.forceReturn`.
- FRT-S02: accept only the current generation's depth-zero frame for the task
  that caused the current non-failure stop. That frame must be an ordinary
  callee with at least one saved caller frame.
- FRT-S03: a value-returning function requires one expression evaluated in the
  callee's lexical frame under the existing detached evaluation policy and
  limits. A procedure accepts no expression and returns `unit`.
- FRT-S04: validate the detached result against a portable declared result
  type retained with function debug metadata before changing live state.
- FRT-S05: success pops exactly the active callee, releases its register
  window, restores the saved caller function/base/instruction pointer, writes
  the result to the saved destination when present, and remains all-stop.
- FRT-S06: success refreshes every stopped-task snapshot once. Failure changes
  no frame, register, task, stop, instruction count, or inspection handle.

## Fixed decisions

- FRT-D01: forced return is a dedicated control-flow operation, not an
  assignment target and not an overload of `setExpression` or `stepOut`.
- FRT-D02: the first slice never selects a non-active frame. Returning an older
  frame would implicitly discard younger frames and requires separate unwind
  semantics.
- FRT-D03: the task responsible for the current stop is the only eligible task.
  Frozen waiting, sleeping, suspended, or merely inspectable peer tasks are not
  rewritten.
- FRT-D04: entry/root frames are rejected. Forced program or task completion
  has termination, retained-result, waiter, and scheduler consequences outside
  this package.
- FRT-D05: runtime-error stops are rejected. This operation is not exception
  recovery and does not suppress an existing diagnostic.
- FRT-D06: the command executes no remaining callee instruction and dispatches
  no synthetic `Return` opcode. The debugger control operation does not add to
  the VM instruction count.
- FRT-D07: ordinary frame teardown owns register release and value drops. No
  callee local, capture, hidden value, or hosted handle is copied into the
  caller except the validated return result.
- FRT-D08: success creates a fresh stop snapshot at the restored caller and
  invalidates stack and variable views. It emits no continued/stopped pair
  because program execution never resumes.
- FRT-D09: `FunctionDebugInfo` gains an optional portable result type. Source
  functions populate it; metadata-less functions remain debuggable for normal
  stepping but cannot be force-returned.
- FRT-D10: Dynamic results, first-class function values, task handles, opaque
  hosted resources, and values rejected by the existing detached policy stay
  unsupported in this first slice.

## Protocol contract

JSONL request:

```json
{"type":"request","id":17,"command":"frame.return","arguments":{"frame_id":4294967297,"expression":"42"}}
```

For procedures, `expression` is omitted. A successful body contains the task
ID, rendered returned value and type, and the fresh caller frame. DAP maps the
same operation from `frameId` and optional `expression`; it returns equivalent
fields and emits one negotiated `invalidated` event covering `stacks` and
`variables` after the response.

## Stable errors

- `unknown_frame`: stale, foreign, or unknown frame ID;
- `frame_return_unsupported`: non-active frame, wrong task, entry/root frame,
  suspended task, runtime-error stop, missing result metadata, or unsupported
  result category;
- `frame_return_value_required`: function request without an expression;
- `frame_return_value_unexpected`: procedure request with an expression;
- `frame_return_type`: evaluated value does not match the declared result type;
- existing parse, evaluation-limit, cancellation, and controlled-call errors
  for the detached expression.

Every failure includes a concrete hint and preserves the complete stopped
state.

## Exit criteria

The package is complete only when all FRT work packages are complete, every
row in `verification-matrix.md` is `PASS`, current user documentation describes
only verified behavior, and `progress.md` records exact successful commands.
