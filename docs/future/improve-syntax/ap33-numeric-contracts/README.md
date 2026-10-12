# AP33: Consistent numeric contracts

Status: Decision pending. Effort: large. Completion is tracked in the
[central plan](../README.md). Existing integer/real behavior is the baseline;
additional fixed-width and decimal types are not authorized by this package.

## Findings

- `1 + 0.5` and `1 < 1.5` work, but `1 = 1.0` and `const R: real := 1` fail.
- Integer addition/subtraction/multiplication wrap; negation/div/mod check
  overflow. `0 - Minimum` and `-Minimum` therefore differ at the lower bound.
- Float arithmetic can produce infinity; nonfinite literals are rejected.
  Division rounding, remainder signs, and NaN behavior need a full contract.
- The minimum signed integer cannot be written as its ordinary negative
  literal because the positive token is checked before negation.
- `RealToStr(1.0)` produces `1`, which `TryReal` rejects. Text underflow yields
  zero while literal underflow is rejected. `TryInt` rejects `$FF` despite
  describing its input as Pascal integer text.

Evidence: [operators](../../../pascal/language/basics/operators.md),
[literals](../../../pascal/language/basics/number-literals.md),
[conversion](../../../pascal/std/text/conv.md), and
[parsing](../../../pascal/std/text/parse.md).

## Proposed direction and open decisions

Choose one compatibility rule across arithmetic, equality, order, assignment,
and arguments. Prefer explicit conversion; uniform limited promotion is an
alternative requiring a decision. Prefer checked ordinary integer arithmetic;
define float exceptional values independently. Decide literal representation,
text grammar, underflow, and a finite-value formatting/parsing round-trip.
Separate exact/checked conversion, rounding, and formatting; `Round` and
`Trunc` are not redundant. Preserve the distinct purpose of bit operations.

## Dependencies and order

AP07/AP18/AP19 supply existing numeric/domain rules. AP33.1 precedes AP33.2
and AP33.3; the latter two implement the same recorded contract. AP43 owns
the public parsing API/Result convention, not the numeric text grammar.

## Work packages

- [ ] [AP33.1: Numeric decisions](01-numeric-decisions.md)
- [ ] [AP33.2: Arithmetic and literals](02-arithmetic-and-literals.md)
- [ ] [AP33.3: Number text](03-number-text.md)

## Acceptance

A normative table predicts result types, conversion, overflow, division,
rounding, and exceptional values. Formatting/parsing and literal boundaries
follow their explicit contracts without depending on incidental host behavior.
