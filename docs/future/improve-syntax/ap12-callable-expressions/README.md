# AP12: Callable expressions

Status: deferred by user decision; retained for possible future reassessment,
with no scheduled implementation. Effort if reopened: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

The retained proposal would make function-valued call targets follow uniform
rules, with actionable diagnostics for non-callable targets. It is not approved
for implementation.

## Decision

Do not implement AP12 now. Direct chained calls such as `MakeAdder(3)(5)` do
not fit the user's preferred Pascal readability. Retain the proposal and its
implementation sketch for possible later use rather than rejecting it permanently.

Existing [anonymous functions and closures](../../../pascal/language/functions/closures.md)
and [named callable values](../../../pascal/language/functions/first-class.md)
remain available. A returned function can be assigned to a named binding before
being called. A factory returning a record with a declared `Add` method can
already use `MakeAdder(3).Add(5)` through
[postfix method chaining](../../../pascal/language/functions/postfix-chaining.md).
That method belongs to the record; this decision adds no `Add` operation to
function values. Indexed callable designators are also already supported.

No current call form is removed. Reopening requires a concrete use case,
an inventory of remaining gaps, and a new explicit user decision on acceptable
call forms and evaluation order.

## Retained proposal (inactive)

- Allow any correctly typed function expression as a call target, including
  returned functions such as `MakeAdder(3)(5)`, indexed values, parenthesized
  values, and function-typed record fields.
- Keep call parentheses explicit. Do not add automatic partial application,
  currying, or a second lambda shorthand; retain existing anonymous functions.
- Calls on function values accept complete positional or named arguments using
  their visible static signature (AP36). Direct returned-callable chaining
  remains outside that implementation and deferred under AP12.
- Preserve current capture storage: captured `const` and value parameters are
  copied; captured local `var` bindings share mutable cells.

## Open decisions

- None for deferral. Before reopening, decide which additional call targets
  are wanted; the retained proposal does not approve all of them.
- Evaluation order: proposed as target (with its indexes) first, then the
  arguments left to right, each exactly once.

## Dependencies

- AP06 (dot calls no longer select callable values by receiver; a callable
  record field is called as a field).

## Order

AP12.1 records deferral. AP12.2 is an inactive sketch; it is not the next work
package and may proceed only after renewed approval.

## Work packages

- [x] [AP12.1: Decision to defer callable expressions](01-callable-expression-decision.md)
- [ ] [AP12.2: Callable expression targets (deferred sketch)](02-callable-expression-targets.md)

## Acceptance

For the current decision: retain the proposal, record deferral, and leave
implemented language behavior unchanged. The uniform-call goal applies only
if the package is reopened and its scope is approved.
