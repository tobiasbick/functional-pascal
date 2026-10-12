# AP43: Consistent error handling and parsing APIs

Status: Decision pending. Effort: medium. Completion is tracked in the
[central plan](../README.md).

## Findings

[Std.Conv](../../../pascal/std/text/conv.md) and
[Std.Parse](../../../pascal/std/text/parse.md) expose the same numeric parser
through aborting StrToInt/StrToReal and Result-returning TryInt/TryReal.
Other fallible APIs use no Try prefix. Option/Result payload handling also
has pattern, predicate-plus-Unwrap, and combinator routes without a clear
canonical selection rule.

The [try page](../../../pascal/language/error-handling/try.md) omits the full
enclosing-return-type/error-compatibility matrix. UnwrapOr evaluates its
fallback argument even on success, unlike conditional/lazy fallback, a
distinction insufficiently highlighted in the native operation references.

## Proposed direction and open decisions

Prefer one Result-returning parser per target type and explicit Unwrap when
aborting is intended. Select naming and whether aborting wrappers are removed.
Define canonical roles: try propagates, is handles one variant with binding,
case handles alternatives, and combinators compose transformations. Decide
whether redundant predicates/helpers remain or are removed; preserve real
semantic differences, including eager versus lazy evaluation and panic versus
recoverable errors. General laziness remains outside the plan.

## Dependencies and work packages

AP33 owns number-text semantics. AP35/AP39 own pattern/case rules. AP43.1 can
evaluate API conventions independently; AP43.2 requires their decisions for
any examples or removals that rely on proposed behavior, and AP33.3 for the
selected numeric parser contract.

- [ ] [AP43.1: Error conventions](01-error-conventions.md)
- [ ] [AP43.2: Error APIs and guidance](02-error-api-and-guidance.md)

## Acceptance

Fallible parsing has one chosen public convention. Documentation specifies
try compatibility, panic separation, and fallback evaluation timing; examples
and diagnostics consistently recommend the chosen handling forms.
