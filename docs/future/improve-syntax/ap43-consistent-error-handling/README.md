# AP43: Consistent error handling and parsing APIs

Status: complete (AP43.1, AP43.2).
Effort: medium. Completion is tracked in the
[central plan](../README.md).

## Implemented behavior

[Std.Parse](../../../pascal/std/text/parse.md) owns Result-returning text parsing;
[Std.Conv](../../../pascal/std/text/conv.md) owns formatting and typed conversion.
The [try page](../../../pascal/language/error-handling/try.md) specifies the full
propagation matrix, generic/nominal error compatibility, and panic separation.
Native Option/Result provide Map, AndThen, OrElse, and Unwrap. Variant checks use
nonbinding is; payload binding, local handling, and plain defaults use exhaustive
case. Callback argument creation remains eager; callback bodies and case arms
run only on their selected variant.

## Agreed contract

The user accepted the decisions recorded in [AP43.1](01-error-conventions.md):

- `Std.Parse` exposes `Int`, `Real`, `Bool`, and `HexInt`, each returning
  `Result of (T, string)`. Remove TryInt/TryReal/TryBool and the aborting
  Std.Conv StrToInt/StrToReal/StrToBool/HexToInt wrappers. Explicit `.Unwrap()`
  expresses deliberate abort. HexInt retains the distinct AP33 hex grammar.
- Use `is` for nonbinding variant tests and `case` for binding payloads, local
  handling, or choosing a plain replacement value. Remove native Option IsSome/
  IsNone, Result IsOk/IsError, and both UnwrapOr helpers.
- Retain Option/Result Map, AndThen, OrElse, and Unwrap. Their roles are wrapped
  transformation, wrapped fallible composition, wrapped recovery, and deliberate
  abort. OrElse retains its existing branch-specific callback behavior.
- Retain the existing `try` matrix: Result propagates into Result with compatible
  error types; Option propagates into Option. Success types may differ.
  Cross-family propagation remains invalid, and panic/runtime failures are not
  caught or converted to Error/None.
- Preserve parsing grammars, numeric bounds, finite-real rules, nominal typing,
  single evaluation, and existing caller effects. An old eager UnwrapOr fallback
  must retain its evaluation when migrated; newly authored `case` fallback work
  belongs in its selected arm.

No selected AP43 language decisions or implementation work remain open.
[AP43.2](02-error-api-and-guidance.md) records implementation ownership, consumer
migration, diagnostics, tooling, regression coverage, and independent follow-ups.
Current language references and generated editor declarations describe this contract.

## Dependencies and work packages

AP33's number-text contract and AP35/AP39's pattern/case rules are complete.
AP43.1 records the agreed convention; AP43.2 delivers its implementation.
Preserve AP34 inference, AP36 complete argument styles, and AP41 qualification.
AP43 replaces selected AP06/AP21 helper entries and AP33 parser names/error
boundaries while retaining their completed status and other delivered contracts.

- [x] [AP43.1: Error conventions](01-error-conventions.md)
- [x] [AP43.2: Error APIs and guidance](02-error-api-and-guidance.md)

## Acceptance

Fallible parsing has one Result-returning operation for each selected input
interpretation. Removed aliases are rejected with the canonical replacement.
Documentation specifies try compatibility, panic separation, helper roles, and
fallback evaluation timing. Migration preserves caller effects and failure
behavior; implementation, consumers, tooling, and guidance agree before AP43
is marked complete. Independent subagent review found no blocking issues.
