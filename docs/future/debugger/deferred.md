# Deferred debugger backlog

This file lists only unresolved capability packages. Implemented behavior and
its supported limits belong in [`docs/pascal/tools/`](../../pascal/tools/debugger.md).
Detailed exclusions may live in an active implementation package, but must not
be duplicated here.

Calls with host I/O, nondeterminism, blocking, tasks, opaque resources, or
unresolved dynamic effects are rejected by the implemented evaluation safety
policy. They are not automatically promises for future implementation.

## Open packages

| ID | Capability package | Deferred boundary | Re-entry gate |
|---|---|---|---|
| DBG-D03 | Remaining identity-bearing assignment | Newly entered or capturing closures, task-bound functions, bound-receiver synthesis, `Dynamic` callable endpoints, task handles, capture cells, opaque resources, or in-place callable editing | Prove portable types, identity, escape, lifetime, ownership, and bounded atomic commit without display-data inference |
| DBG-D04 | Control flow beyond selected-frame return | Root/task completion, completed-return replacement, runtime-error recovery, arbitrary instruction changes, source-initializer suppression, and frame restart | Specify scheduler and waiter effects, frame/task ownership, portable result proof, cleanup behavior, and rollback |
| DBG-D05 | Task control and history | Non-stop execution, per-task resume/pause, task creation/cancellation/restart, scheduler shortcuts, retained completed-task state, and task-history UI | Define a quiescence protocol, shared-state visibility, cancellation propagation, retention/privacy bounds, and protocol-equivalent stops |
| DBG-D06 | Interactive hosted programs | Live terminal input, full-screen console/TUI debugging, graph-window events while paused, and reliable pause inside blocking host calls | Separate debuggee transport from DAP stdio and prove authentication, cleanup, input ordering, cancellation, and host-event behavior |
| DBG-D07 | Attach, remote, and native debugging | Attach to a running VM or bundle, remote sessions, and OS-level machine-code debugging | Define discovery, authentication, version negotiation, source mapping, disconnect ownership, and a recoverable transport |
| DBG-D08 | Advanced breakpoints | Data and function breakpoints, richer runtime-failure filters, and breakpoint actions that mutate state or control flow | Define stable runtime identities, exact stop/mutation semantics, bounded overhead, atomicity, and JSONL/DAP parity |
| DBG-D09 | Reverse execution and hot reload | Backward stepping, deterministic recording/replay, suspended code replacement, and preserving live frames across recompilation | Define a versioned VM/host snapshot boundary and compatibility rules for active functions, layouts, values, tasks, and external effects |

## Maintenance rules

- Keep one row per independently specifiable unresolved package.
- Completing a package removes its row; current limitations stay in user-facing
  documentation and regression tests.
- Splitting a package replaces its parent row. Do not append children while
  retaining the same parent description.
- Active plans use stable work IDs, evidence, resume instructions, and their
  own consciously-deferred file. On completion, retain only genuinely
  independent unresolved packages here.
- Do not use this backlog to preserve implementation history.

The next debugger package should select one bounded row, create its own plan
under this directory, and leave every unrelated row unchanged.
