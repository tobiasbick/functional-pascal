# AP26.3: Task handle escape restrictions

Package: [AP26: Structured task scopes](README.md)

## Scope

Reject task handles of scope-owned children escaping their scope by return,
storage in an outer variable, or channel send (Q20).

## Prerequisites

- AP26.1 (scope ownership).
- The closure-capture limits in the [package README](README.md#open-decisions).

## Implementation

- Sema: track scope provenance of task handles through bindings, containers,
  closures, returns, outer assignments, and channel sends; reject escapes with
  a diagnostic naming the owning scope.
- Specify capture limits for closures that hold scope-owned handles,
  consistent with these rules.
- Unit interfaces carry what is needed to check helpers that receive handles.

## Affected areas

- `crates/fpas-sema/src/check/expr/task_bound.rs`, `check/closures/`,
  `fpas-unit` interfaces.

## Migration

Adapt any repository code inside new scopes that stores handles outside them.

## Documentation

- The task-scope page, `task-handles.md`.

## Verification

- Rejected escapes through return, outer variables, containers, closures, and
  channels; accepted local use; nested scope provenance; imported helpers.
