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

## Reference

The reference branch `codex/syntax-changes` implemented callable targets after
any typed expression with target-then-arguments evaluation, reusing the
existing `CallValue` instruction. It also found and fixed a capture defect for
captured mutable parameters (the former F9001 identity). AP16.3 has removed
that parameter mode and migrated captures to local `var` copies; those captures
have regression coverage. AP12 must preserve the current local-copy behavior.
