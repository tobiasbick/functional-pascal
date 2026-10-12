# AP33: Consistent numeric contracts

Status: complete. Effort: large. Completion is tracked in the
[central plan](../README.md). The package covers the existing integer and real
types and public numeric APIs.

## Selected contract

The user selected explicit numeric compatibility, checked integer arithmetic,
finite real results, and shared numeric literal/text forms. The complete
decision table is in [AP33.1](01-numeric-decisions.md).

| Concern | Selected rule |
| --- | --- |
| Numeric compatibility | No implicit integer/real mixing in arithmetic, comparison, assignment, arguments, or generic inference. Use `IntToReal` explicitly. |
| Result types | Integer `+`, `-`, `*`, `div`, and `mod` return integer; real arithmetic returns real. `/` on two integers or two reals always returns real, including generic Numeric operands of the same type. |
| Integer boundaries | Ordinary addition, subtraction, multiplication, and negation are checked. The direct signed minimum literal is valid; values outside signed 64-bit bounds are rejected. |
| Integer division | `div` truncates toward zero; `mod` has the dividend's sign. Zero divisors fail. Minimum divided by -1 fails; minimum modulo -1 is zero. |
| Real values | NaN and infinity are rejected as operands and results. Division by either sign of zero fails. Arithmetic follows binary64 rounding and gradual underflow; signed zero is preserved. |
| Conversion | `IntToReal` performs explicit binary64 rounding, which may lose integer precision. `Round` uses ties to even; `Trunc` truncates toward zero. Integer results are range-checked. |
| Literal/text grammar | Shared decimal, `$` hexadecimal, and single underscore separators; real forms require digits on both sides of the decimal point and permit an exponent. Input text also permits surrounding whitespace and an optional sign. Nonzero literal/text underflow to zero fails. |
| Formatting | Finite real formatting is locale-independent and round-trips to the identical binary64 value, including integral reals and signed zero. |
| Existing type identities | Subrange arithmetic retains its integer behavior; distinct types retain explicit wrap/unwrap boundaries. Std.Bits keeps its separate bit semantics. |

Evidence: [operators](../../../pascal/language/basics/operators.md),
[literals](../../../pascal/language/basics/number-literals.md),
[conversion](../../../pascal/std/text/conv.md), and
[parsing](../../../pascal/std/text/parse.md).

## Implementation boundary

The selected rules apply across checking, constant evaluation, lowering,
optimization, runtime execution, debugger evaluation, and standard numeric
operations. The shared [numeric core](../../../../crates/fpas-numeric/src/lib.rs)
owns checked arithmetic and literal/input grammar. Error-producing operations
remain on executed paths; unselected branches and short-circuit operands remain
unevaluated. Bytecode version 20 prevents reuse of compiled artifacts with
superseded arithmetic behavior.

## Dependencies and order

AP07/AP18/AP19 supply existing numeric/domain rules. AP33.1 precedes AP33.2
and AP33.3; the latter two implement the same recorded contract. AP43 owns
the public parsing API/Result convention, not the numeric text grammar.

## Work packages

- [x] [AP33.1: Numeric decisions](01-numeric-decisions.md)
- [x] [AP33.2: Arithmetic and literals](02-arithmetic-and-literals.md)
- [x] [AP33.3: Number text](03-number-text.md)

## Acceptance

A normative table predicts result types, conversion, overflow, division,
rounding, and exceptional values. Formatting/parsing and literal boundaries
follow their explicit contracts without depending on incidental host behavior.

## Regression coverage

Compiler execution tests cover overflow, finite real results, rounding,
generic numeric operations, arithmetic underflow, signed minimum literals,
division/remainders, and unevaluated paths. Sema tests cover numeric mixtures
at operators and value boundaries. Parser and shared text tests cover grammar,
range diagnostics, and bit-preserving finite real round-trips. VM tests retain
raw instruction checks; build tests verify exported defaults through reused
compiled units. FPAS tests exercise public conversion, parsing, predicate, and
TOML behavior through the installed unit graph.

## Open decisions

None within this package's selected scope.
