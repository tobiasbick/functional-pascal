# AP23: Preconditions and postconditions

Status: complete (AP23.1 and AP23.2).
The direction is agreed (Q15, Q16, Q17). Effort: large. Completion is tracked
in the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Contracts are typed and restricted to side-effect-free expressions; checks
remain active in release builds, violations panic with the agreed details, and
contracts cannot substitute for a missing return.

## Decisions

### Keywords (Q15)

- Use `requires` for preconditions and `ensures` for postconditions.
- Both become reserved words.
- Multiple clauses of the same kind are combined with logical `and`.

### Return-value naming (Q16)

- An `ensures` clause may name the return value explicitly, for example
  `ensures (Clamped) Clamped >= Lower and Clamped <= Upper;`.
- The name denotes the routine's return value and is visible only in that
  clause. It does not introduce a binding in the body or in other clauses.
- Do not use the routine name or `return` as a special return-value reference.
  `Result` remains reserved for the existing result type.

### Checking and violations (Q17)

- Contracts are always checked, including release builds. No build mode
  disables them.
- Check `requires` on routine entry and `ensures` on every return.
- A violation causes a panic identifying the routine, clause text, and
  parameter values; it is not a `Result` error.
- Initially provide no `old` values or entry-state snapshots.
- Contract expressions may use parameters, constants, operators, and built-in
  length/membership checks; postconditions may also use their named return
  value. Ordinary function and method calls remain unavailable in clauses.
  AP25 was closed without a purity extension.
- No automatic proofs.

Contract heading fragment; the body is omitted:

```pascal
function Clamp(Value: integer; Lower: integer; Upper: integer): integer;
requires Lower <= Upper;
ensures (Clamped) Clamped >= Lower and Clamped <= Upper;
```

### Expression and binding boundary

- Permit parameters, including `Self` and the routine's own `var` parameters,
  and visible initialized immutable values: static or computed `const`
  bindings and immutable enclosing parameters/captures.
- Reject reads of mutable globals and mutable captures. Body-local bindings
  are outside the heading clauses' scope.
- Permit literals, operators, field/index reads, explicit conversions, typed
  construction, and `if`/`case` expressions. Apply the same restrictions to
  every subexpression, including arguments, guards, and omitted field defaults.
- The native-call allowlist is `Length`, `IsEmpty`, `Contains`, and
  `ContainsKey`, on their existing string, array, or dictionary receiver types.
  Resolve the actual native operation; a user routine with the same name is
  not an allowed call. Membership includes the `in` operator on values and
  subrange types.
- Reject ordinary function/method calls, callable-value calls, `try`, `go`,
  and anonymous routines inside clauses. No contract expression may propagate
  a return, spawn work, or introduce a callable capture.
  Initialized callable constants and parameters remain readable as values;
  forming a routine value or a bound method inside a clause is rejected.

The current [constant classification](../../../pascal/language/basics/constants.md)
distinguishes computed immutable bindings from compile-time constants.
[Membership](../../../pascal/language/basics/operators.md#comparison) includes
values and subrange types. The current [`is` test](../../../pascal/language/pattern-matching/is-test.md#where-is-is-valid)
retains its existing condition positions; AP23 does not extend its scope.

### Propagated returns (AP23.2)

- Run postconditions on ordinary returns and `try` error exits alike.
- A clause-local return-value name denotes the complete declared return type,
  including `Error(...)` for `Result` and `None` for `Option`, not an unwrapped
  success payload.
- Observe current parameter values; do not snapshot entry state or roll back
  writes. Existing
[`var` parameter semantics](../../../pascal/language/functions/var-parameters.md#evaluation-and-writes)
  preserve completed writes even if a postcondition panics.

## Open decisions

None for AP23.1 or AP23.2.

## Dependencies

- AP07 (boolean rules for clause expressions).
- AP13 (routine declaration syntax with named closers).
- AP16 (constant classification for clause expressions).

AP25's reassessment is complete; the package is closed without implementation.

## Order

AP23.1 implements preconditions and shared expression checks. AP23.2 implements
postconditions with a clause-local return name and checks every ordinary or
propagated return. Both are documented under
[preconditions and postconditions](../../../pascal/language/functions/contracts.md).

## Work packages

- [x] [AP23.1: Preconditions](01-preconditions.md)
- [x] [AP23.2: Postconditions](02-postconditions.md)

## Acceptance

Contracts are typed and restricted to side-effect-free expressions; checks
remain active in release builds, violations panic with the agreed details, and
contracts cannot substitute for a missing return.
