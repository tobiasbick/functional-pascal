# AP12: Callable expressions

Status: proposal. Requires an explicit decision before language
implementation (AP12.1). Effort: medium. Completion is tracked in the
[central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

All function-valued call targets follow the same rules; non-callable targets
receive an actionable diagnostic.

## Proposal

- Allow any correctly typed function expression as a call target, including
  returned functions such as `MakeAdder(3)(5)`, indexed values, parenthesized
  values, and function-typed record fields.
- Keep call parentheses explicit. Do not add automatic partial application,
  currying, or a second lambda shorthand; retain existing anonymous functions.
- Calls on function values are positional only (AP09).
- Preserve current capture storage: captured `const` and value parameters are
  copied; captured local `var` bindings share mutable cells.

## Open decisions

- Accept or reject the proposal.
- Evaluation order: proposed as target (with its indexes) first, then the
  arguments left to right, each exactly once.

## Dependencies

- AP06 (dot calls no longer select callable values by receiver; a callable
  record field is called as a field).

## Order

AP12.1 records the decision; AP12.2 implements it.

## Work packages

- [ ] [AP12.1: Callable expression decision](01-callable-expression-decision.md)
- [ ] [AP12.2: Callable expression targets](02-callable-expression-targets.md)

## Acceptance

All function-valued call targets follow the same rules; non-callable targets
receive an actionable diagnostic.
