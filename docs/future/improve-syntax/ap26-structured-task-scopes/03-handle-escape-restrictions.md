# AP26.3: Task handle escape restrictions

Package: [AP26: Structured task scopes](README.md)

## Scope

Reject task handles of scope-owned children escaping their scope (Q20, Q27).

## Prerequisites

- AP26.1 (scope ownership).

## Implementation

- Sema: track the owning scope of values derived from scope-owned `go`
  expressions through bindings, aggregates, and calls. Accept them in
  bindings declared inside the owning scope, in `Std.Tasks` wait operations,
  and as arguments of direct routine calls whose parameter does not escape.
  Reject return, assignment to bindings declared outside the scope (including
  `var` parameters, fields, and elements), channel send, capture by an
  anonymous routine, and calls through callable values, with a diagnostic
  naming the owning scope.
- Infer per parameter whether it can escape, using the same sinks, and record
  the result in the unit interface for imported helpers.
- Loop-carried origins reach a stable state. Named nested routines reuse the
  semantic capture metadata as implicit analysis inputs; their escape results
  contribute to the enclosing routine's parameter contract. Declaration
  initializers are checked in programs and units as well as routine bodies.
- Native callback operations reject scope handles passed through callable
  values, including implicit callback inputs and retained callback captures.
- No runtime fallback.

## Affected areas

- `crates/fpas-sema/src/check/` (a focused scope-escape module),
  `fpas-unit` interfaces.

## Migration

Adapt any repository code inside new scopes that stores handles outside them.

## Documentation

- The task-scope page, `task-handles.md`.

## Verification

- Rejected escapes through return, outer variables, `var` parameters,
  containers, closures, callable values, and channels; accepted local use,
  local containers, and wait operations; helpers with non-escaping and
  escaping parameters, including imported helpers; nested scope provenance.

Regression ownership: sema statement tests cover escape routes, loop-carried
origins, nested captures, native callbacks, and closure initializers. Sema
interface tests encode and decode the dependency interface before checking
aliased imported helpers and record methods, and verify that changed escape
contracts change the interface digest.
