# UMB-20 scope and fixed decisions

## Authorized scope

- `UMB-20A`: source-independent function breakpoints.
- `UMB-20B`: runtime-failure stop filters based on stable diagnostics.
- `UMB-20C`: bounded, non-mutating condition, hit-count, and log behavior that
  is meaningful for the breakpoint kinds supported by each protocol.
- Shared VM/session behavior plus JSONL, DAP, and VS Code adapter coverage.

No FPAS syntax, semantics, bytecode execution semantics, or language-spec page
may change in this package.

## `UMB-20A` — function identity and matching

1. The request name is a selector; the bound identity is an ordered set of
   exact `FunctionId` values and executable entry sequence points.
2. Reuse `vm::debug::routines::matching_functions`. Matching remains
   case-insensitive and accepts a canonical name or its established short-name
   suffix.
3. A selector matching multiple routines binds all exact matches in ascending
   `FunctionId` order. The adapter reports the match count; it must not pick a
   routine by source order or display name.
4. A matching routine without an executable entry sequence point does not
   produce a physical binding. A logical breakpoint with no physical binding
   remains unverified and explains why.
5. Setting the collection is replace-all and atomic. A rejected request leaves
   the old collection and hit counters unchanged.
6. A single logical breakpoint ID is returned when any of its physical entry
   bindings stops. Recursive calls and calls from any task count as hits of
   that logical breakpoint.
7. Source and function breakpoints may share an instruction. Their logical IDs
   are evaluated in ascending session ID order and yield at most one stopped
   event for that boundary.

## `UMB-20B` — runtime-failure filters

1. The default remains `all`: every runtime `Diagnostic` produces an
   inspectable exception stop as today.
2. Exact stable diagnostic codes are the initial filter identity. The protocol
   accepts only codes present in the central diagnostic catalog and rejects an
   unknown code atomically.
3. Categories may be exposed only if `fpas-diagnostics` gains a central,
   exhaustive category taxonomy. Do not infer categories from message text or
   numeric gaps.
4. A nonmatching runtime failure still emits its structured diagnostic and
   ends the debuggee with failure. It skips only the inspectable stopped state;
   it can never continue the failed instruction.
5. DAP uses `setExceptionBreakpoints` and advertised exception filters. JSONL
   exposes an equivalent replace-all filter command with the same validation.
6. Filter state is session-local and survives ordinary continue/step commands,
   but not a new debugger session.

## `UMB-20C` — non-mutating policies

1. Function breakpoints support condition and exact physical hit count where
   the protocol represents them. DAP `FunctionBreakpoint` does not gain a
   custom log-message field.
2. Existing source logpoints remain supported. Any new JSONL-only action must
   first have an explicit shared-engine representation and a documented DAP
   or editor outcome; otherwise it is rejected as adapter divergence.
3. When several logical breakpoints share a boundary, evaluate them in session
   ID order. Emit bounded log output before the single stopped event.
4. Evaluation errors are bounded and deterministic and must not partially
   replace breakpoint state or mutate debuggee values.
5. State-changing actions, forced returns, expression assignment, task
   control, and data mutation remain in `UMB-70C` or their existing package.

## Required resource bounds

Before `U20-01` becomes done, record constants for maximum logical function
breakpoints, maximum physical bindings per logical breakpoint, maximum failure
filters, condition length, log-template length, and emitted log bytes. Existing
source-breakpoint limits should be reused when they cover the same resource.
