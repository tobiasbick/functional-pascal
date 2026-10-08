# AP23: Preconditions and postconditions

Status: agreed direction (Q15, Q16, Q17). Effort: large. Completion is tracked
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
  value. Further function calls require a `pure` annotation once AP25 is
  implemented. AP23 does not depend on that later extension.
- No automatic proofs.

Draft contract fragment; the body is omitted:

```pascal
function Clamp(Value: integer; Lower: integer; Upper: integer): integer;
requires Lower <= Upper;
ensures (Clamped) Clamped >= Lower and Clamped <= Upper;
```

## Open decisions

Before AP23.2, decide whether a `try` error return runs postconditions and how
the clause-local return-value name applies to that path. AP23.2 requires
this rule; Q17 does not explicitly distinguish ordinary and propagated returns.

## Dependencies

- AP07 (boolean rules for clause expressions).
- AP13 (routine declaration syntax with named closers).
- AP16 (constant classification for clause expressions).

AP25 is reassessed after practical use of this package.

## Order

AP23.1 delivers preconditions with the shared clause machinery; AP23.2 adds
postconditions with the named return value.

## Work packages

- [ ] [AP23.1: Preconditions](01-preconditions.md)
- [ ] [AP23.2: Postconditions](02-postconditions.md)

## Acceptance

Contracts are typed and restricted to side-effect-free expressions; checks
remain active in release builds, violations panic with the agreed details, and
contracts cannot substitute for a missing return.
