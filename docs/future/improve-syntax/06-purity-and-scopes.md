# Purity and scopes

See the [steering document](README.md). These are proposals, not current APIs.
An effect system is excluded from this plan; see the steering document.

## AP25: Conservative purity

### Decided direction (Q18)

- Retain this package at low priority. Reassess implementation after AP23
  contracts have been used in practice; their initial form does not require
  this checker.
- Support `pure function` only, not pure procedures. Pure functions have no
  `var` parameters and may call only other pure functions.
- Captures must be immutable. Local `var` bindings are allowed only when their
  mutation cannot be observed externally.
- Panics are allowed; purity does not imply termination or freedom from errors.
- Standard-library routines carry purity metadata for checked calls.

### Tasks

- [ ] Reassess implementation priority after practical use of AP23.
- [ ] Define an initial conservative `pure function` contract: no I/O,
  observation of globally mutable state, or externally observable mutation.
- [ ] Enforce pure-only calls and the prohibition of pure procedures and `var`
  parameters; provide verified purity metadata for standard-library routines.
- [ ] Specify immutable captures, aliasing, and imported API behavior.
- [ ] Allow local mutable bindings only if mutation cannot be observed externally;
  explain conservative restrictions rather than treating immutable bindings as proof.
- [ ] Describe allowed side effects of ordinary functions/procedures. Purity
  proves neither termination nor freedom from errors.
- [ ] Test positive/negative calls, captures, aliasing, and shared state;
  rejected pure procedures and `var` parameters; allowed local mutation and
  panics; and standard-library purity metadata.

Acceptance: the checker enforces an explicit, explainable boundary and cannot
infer purity merely from immutable variable names.

## AP26: Structured task scopes

Builds on the existing `go` and `task of T` model; it does not introduce
deferred workflows.

### Decided completion rule (Q19)

- On normal arrival at `end scope;`, wait for all child tasks to finish without
  requesting cancellation.
- On any other exit from the scope (`return`, `break`, error propagation, or
  panic), request cancellation of the remaining children and then wait for
  their completion before leaving the scope.
- A failed child also triggers cancellation of the remaining children followed
  by waiting for completion, using the failure propagation rules below.

### Decided ownership and failures (Q20)

- Every `go` inside a scope starts a child owned by the innermost scope,
  whether used as an expression or a statement. Its handle cannot escape by
  return, storage in an outer variable, or channel send.
- A `go` outside any scope remains detached.
- The first child failure requests cancellation of its siblings. After all
  children finish, the scope ends with that failure: re-raise a panic, or
  propagate a `Result` error if the enclosing routine returns a compatible
  `Result`. Otherwise convert the error to a panic. Attach further failures
  to the diagnostic.
- Keep `Std.Tasks` groups for dynamic cases such as passing groups between
  routines or supervised retries. A scope is not a user-visible task group;
  internal mechanisms may be reused where their contracts fit.
- Reserve `scope` as a keyword.

```pascal
scope
  const A: task of integer := go Fetch(1);
  const B: task of integer := go Fetch(2);
  Show(Wait(A) + Wait(B));
end scope;
```

### Tasks

- [ ] Implement `scope ... end scope;` with normal waiting and cancellation
  followed by waiting on abnormal completion, as specified above.
- [ ] Implement first-failure propagation, compatible `Result` forwarding,
  panic fallback, and diagnostics containing further failures.
- [ ] Enforce innermost-scope ownership and handle escape restrictions;
  specify capture limits consistent with those rules.
- [ ] Reserve `scope` and diagnose its use as an identifier.
- [ ] Relate the rules to existing task groups and cancellation in `Std.Tasks`;
  reuse them where their verified contracts fit.
- [ ] Test one child task first, then every exit path: normal completion,
  `return`, error propagation, `break`, panic, and child failure. Verify that
  normal completion does not cancel unfinished work and that no exit leaves
  owned children running after the scope has ended.
- [ ] Prevent switching a `go` expression to a statement from silently changing
  its lifetime.
- [ ] Test nested ownership, rejected handle escapes through returns, outer
  variables and channels, detached tasks outside scopes, both `go` forms,
  sibling cancellation, multiple failures, compatible `Result` propagation,
  and panic fallback.

Acceptance: ownership, detachment, and completion are visible and governed by
the same explained rule on every exit path.
