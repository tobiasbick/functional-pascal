# AP36: Consistent calls and construction

Status: Decision pending. Effort: large. Completion is tracked in the
[central plan](../README.md). AP09/AP10 remain complete under their existing
rules until an explicit replacement decision is recorded.

## Findings

The [argument rules](../../../pascal/language/functions/parameters.md)
allow `Double(Value := 3)` but reject `F(Value := 3)` after assigning Double
to a callable whose written type includes the parameter name. Callable
parameter names are required syntactically but do not survive as call labels.
Variadic/polymorphic builtins add further positional-only exceptions.

[Records](../../../pascal/language/types/records.md) require named fields;
[enum construction](../../../pascal/language/types/enums.md) permits either
positional or named fields, while enum patterns only permit positional fields.

## Proposed direction and open decisions

Choose a single routine-call convention, or a justified consistent role for
both styles. Positional-only routine calls reduce forms but lose named-role
clarity; retaining named calls requires deciding whether labels belong to
callable contracts and how differently named signatures remain compatible.
Do not erase AP09's readability benefit without weighing it explicitly.

Consider named-only user aggregate construction to align records and enums.
Decide pattern syntax independently, and specify built-in single-payload
Some/Ok/Error rather than leaving accidental exceptions. No new callable
expression syntax is proposed; AP12 remains deferred.

## Dependencies and work packages

AP09/AP10/AP20 are complete. AP36.1 precedes AP36.2. AP34 owns type inference;
neither package may change the other's rule implicitly.

- [ ] [AP36.1: Argument conventions](01-argument-conventions.md)
- [ ] [AP36.2: Unified call rules](02-unified-call-rules.md)

## Acceptance

Extracting a routine into a callable value follows one predictable argument
contract. Construction and pattern rules each have a canonical form, and
the decision preserves or explicitly changes argument evaluation order.
