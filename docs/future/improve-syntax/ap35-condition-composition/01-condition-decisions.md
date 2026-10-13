# AP35.1: Condition decisions

Package: [AP35](README.md). Status: complete; decision work package.

## Recorded contract

The user approved the [package decisions](README.md#recorded-decisions):

- `is` is a nonbinding Boolean expression in every ordinary Boolean context.
  Parentheses preserve its meaning; Boolean operators keep their existing
  precedence, grouping, and evaluation rules.
- `case` arms are the sole place to introduce pattern-bound names. An `is`
  pattern containing `const Name`, including a nested binding, is invalid.
- Return-value naming uses `ensures (Name): Expression;`. A leading grouped
  expression without the colon is a condition, not a binder. Procedure clauses
  have no return binder; a function's result name remains local to its clause.

The decision explicitly replaces AP20.3's binding and condition-only `is`
model and AP23/Q16's colon-free binder spelling through their implementation
owners. Other pattern rules, case coverage, return timing, contract operand
restrictions, and AP23's native-operation allowlist are preserved.

## Accepted and rejected forms

The table describes the implemented contract. Existing declarations supply the
referenced Option values, Boolean parameters, and routines.

| Form | Result |
|-------------|--------|
| `if Value is Some(_) then ...` | Accepted nonbinding condition. |
| `if (Value is Some(_)) then ...` | Accepted with identical meaning. |
| `not (Value is Some(_))` | Accepted Boolean negation. |
| `(Value is Some(_)) or Enabled` | Accepted Boolean composition. |
| `while Value is Some(_) do ...` | Accepted; re-evaluated per iteration. |
| `repeat ... until Value is None;` | Accepted Boolean loop condition. |
| `const Present: boolean := Value is Some(_);` | Accepted Boolean value. |
| `when Some(const V) if Other is None: ...` | Accepted case binding and nonbinding guard. |
| `Value is Some(const V)` | Rejected in every context; bind V in a case arm. |
| `Value is Ok(Some(const V))` | Rejected nested binding. |
| `ensures (Returned): Returned > 0;` | Accepted function result binder. |
| `ensures (Flag);` | Accepted ordinary Boolean condition in a function or procedure. |
| `ensures (Flag and Ready);` | Accepted ordinary grouped condition. |
| `ensures (Returned) Returned > 0;` | Rejected legacy binder spelling. |
| `ensures (Returned): Returned > 0;` in a procedure | Rejected: a procedure has no return value. |

Other restrictions still apply to these fragments: case exhaustiveness, allowed
pattern constants, declared types and names, and the contract expression rules.

## Ownership and prerequisites

AP20/AP23 provide the implemented foundation. [AP35.2](02-pattern-conditions.md)
owns Boolean pattern tests and migration of binding tests;
[AP35.3](03-postcondition-binding.md) owns the binder grammar and migration.
Both implementation work packages are complete. The handbook, formal grammar,
and compiler describe this contract.

## Verification

The decision records expression contexts, rejection of all pattern bindings
inside `is`, the single binder spelling, preserved behavior, and migration
owners. No language decision remains open within this package's scope.
Executable regressions and current handbook updates are recorded in the
completed AP35.2 and AP35.3 packages.
