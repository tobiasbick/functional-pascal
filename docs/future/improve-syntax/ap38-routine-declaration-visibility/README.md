# AP38: Routine declaration visibility

Status: Decision pending. Effort: medium to large. Completion is tracked in
the [central plan](../README.md).

## Finding

[Type/member signatures](../../../pascal/language/types/declaration-order.md)
are available throughout a declaration scope, but free routines are available
only after their declarations. Record method bodies and defaults still see
free routines at the record's source position. Moving a helper into a static
method changes forward-reference availability; mutual recursion can require
structural workarounds.

## Proposed direction and open decisions

Prefer collecting all named routine signatures in a lexical declaration scope
before checking bodies, while preserving ordered value initialization.
Decide nested scopes, mutual recursion, defaults/initializers invoking later
routines, and reads of not-yet-initialized values. Source-order-only routines
remain an alternative if the additional initialization model is not wanted.

## Dependencies and work packages

AP11/AP24 supply forward type/member handling. AP38.1 precedes AP38.2.
No dependency on AP12 or a new callable-expression form exists.

- [ ] [AP38.1: Visibility decision](01-visibility-decision.md)
- [ ] [AP38.2: Routine signatures](02-routine-signatures.md)

## Acceptance

One lexical signature-visibility rule explains methods and free routines.
Forward calls cannot expose uninitialized values, and the chosen recursive
call behavior works across nested scopes and unit interfaces.
