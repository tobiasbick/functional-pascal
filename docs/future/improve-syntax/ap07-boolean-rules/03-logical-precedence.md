# AP07.3: Logical precedence and mixing rules

Package: [AP07: Boolean rules](README.md)

Status: complete.

## Result

Comparisons bind above `not`, which binds above binary logical operators.
Chains of one logical operator are valid; mixing `and`, `or`, and `xor` requires
explicit parentheses. Comparisons are non-associative; `A < B < C` is rejected
with a hint such as `A < B and B < C`.

Parser, formatter and debugger use the same table. Formatting preserves
explicit grouping and adds required grouping for generated expressions.
The planned `is` pattern test belongs to AP20.3.

## Regression coverage

Parser, formatter, CLI and debugger tests cover every boundary, mixed operator
pairs, negation, comparison chains, recovery, wrapping and round trips.
See the [precedence table](README.md#precedence-table-q03).
