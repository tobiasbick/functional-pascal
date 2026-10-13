# AP39.1: Coverage decision

Package: [AP39](README.md). Status: complete; agreed contract recorded.
The selected contract is implemented by
[AP39.2](02-coverage-checks.md).

## Recorded decision

The user selected always-exhaustive `case`, with identical rules for statements
and expressions. Use `if` for selective handling. The selector is evaluated
once; labels and arms are tried in written order, and the first matching label
whose guard succeeds selects its arm. Overlapping labels in one arm may evaluate
the guard again after a failure. AP35's guard and binding contexts remain intact.

| Selector | Coverage requirement | `else` |
|----------|----------------------|--------|
| Enum, Option, Result | Every variant explicitly, with recursive payload coverage | Forbidden, including after complete coverage |
| Boolean | Both true and false explicitly | Forbidden |
| Subrange | All declared values, using values or supported ranges | Forbidden |
| Integer, string | A fallback after explicit labels or guarded arms | Required |
| Distinct | Same coverage as its supported underlying type, using typed labels | Same as the underlying type |

Aliases retain the target type's rules. Distinct Boolean selectors require
both typed Boolean values; distinct integer/string selectors require `else`.
This does not broaden the existing selector types: real and distinct real
remain unsupported. Integer is treated as an open domain for case coverage;
even a machine-wide integer range does not replace its required `else`.

## Nested patterns, guards, and reachability

Coverage applies recursively to the existing nested patterns. An ignored or
bound payload covers its entire position, for example `Some(_)` or
`Some(const Value)`. Nested Boolean and subrange values contribute their actual
finite coverage; nested open scalar values still need a covering payload
pattern. There is no nested `else` syntax or new range-pattern syntax.
Whole-selector catch-alls do not replace the matrix's required handling.
Scalar `const Name` labels retain their existing requirement for a guard.

Guards never contribute coverage or make later labels unreachable, even when
a guard is the constant true. Do not reason about Boolean equivalence or
complementary guards. A guarded label can itself be unreachable when earlier
unguarded labels already cover all values it can match.

Reject duplicate labels within one arm and labels fully covered by earlier
unguarded arms, for scalar and variant cases alike. Compare compile-time
matched values rather than spelling, including named constants, typed conversions,
singleton value/range forms, and ranges equivalent within the selector domain.
Full coverage may come from the union of multiple earlier labels or ranges.
Separate guarded arms may repeat a label, because a failed guard permits
the next arm to match. Partial overlap remains legal and follows written order;
no separate prohibition for overlapping ranges is introduced.

Reason only about declared variants, supported patterns, compile-time values,
and supported ranges. Use exact interval coverage for subranges without
enumerating their values; do not introduce a size cutoff that changes the
language rule. Unsupported or runtime-valued labels retain their existing
type/constant diagnostics. Unknown guard behavior never proves coverage.

## Decision example matrix

These are implemented outcomes, covered by AP39.2's boundary regressions.
They apply equally to statements and expressions.

| Selector and arms | Selected outcome |
|-------------------|------------------|
| Boolean: only true | Reject: false is missing |
| Boolean: true plus else | Reject: else is forbidden; write false |
| Boolean: true and false | Accept without else |
| Subrange 1..3: labels 1, 2, 3 | Accept without else |
| Subrange 1..3: ranges 1..2 and value 3 | Accept without else |
| Subrange 1..3: labels 1, 3 | Reject: value 2 is missing |
| Subrange 1..3: full coverage plus else | Reject: else is forbidden |
| Integer/string: selected values without else | Reject: else is required |
| Integer/string: selected values plus else | Accept |
| Distinct Boolean: typed true and typed false | Accept without else |
| Distinct integer/string: typed labels without else | Reject: else is required |
| Option of Boolean: Some(true), None | Reject: Some(false) is missing |
| Option of Boolean: Some(true), Some(false), None | Accept |
| Option of Boolean: Some(_), None | Accept |
| Boolean: true with a constant-true guard, false unguarded | Reject: true lacks unguarded coverage |
| Integer: duplicate value labels in one arm, even with a guard | Reject: duplicate label |
| Integer: value 1 unguarded, then value 1 with or without a guard | Reject: later label is unreachable |
| Integer: value 1 guarded, then value 1 unguarded, plus else | Accept: guard failure reaches the second arm |
| Integer: range 1..5, then range 3..8, plus else | Accept: the second range still matches 6..8 |
| Integer: ranges 1..2 and 3..5, then range 2..4, plus else | Reject: the later range is fully covered |
| Result: Ok(0), Ok(const Value), Error(const Message) | Accept: the second arm handles other Ok values |
| A new enum variant is added without a corresponding arm | Reject: new variant is missing |
| A subrange is extended beyond the explicit labels/ranges | Reject: newly admitted values are missing |

Partial overlap supports the normal special-case/fallback structure:

```pascal
case Response of
  when Ok(0):
    HandleZero();
  when Ok(const Value):
    HandleValue(Value);
  when Error(const Message):
    HandleError(Message);
end case;
```

## Migration and documentation boundary

AP03/AP18/AP19/AP20/AP21 provide the implemented baseline. AP39 replaces their
scalar statement/expression coverage distinctions while preserving explicit
closed variants, pattern scope, type identity, and completed package status.
The decision requires explicit no-op arms where a complete case intentionally
does nothing. Prefer an existing `if` or nonbinding `is` test for selective
handling; preserve evaluation count and effects during migration.

The current [exhaustiveness reference](../../../pascal/language/pattern-matching/exhaustiveness.md)
and related language pages describe these rules. No contract decision remains
open. AP39.2 delivers compiler checks, consumer
migration, diagnostics, editor guidance, documentation, and regression coverage.
