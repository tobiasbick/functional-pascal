# AP33.1: Numeric decisions

Package: [AP33](README.md). Status: Decision pending; decision work package.

## Scope and prerequisites

Inventory current operators, literal parsing, conversion functions, constant
evaluation, and generic Numeric constraints. AP07/AP18/AP19 are complete.
Use the package's counterexamples to compare explicit conversion with uniform
promotion; do not infer approval from the earlier feasibility discussion of
fixed-width types, decimal, or graphics buffers.

## Deliverable and affected areas

Record the user's selected compatibility and overflow rules, signed minimum
literal handling, div/mod behavior, float zero/NaN/infinity behavior, text
syntax and bounds, and formatting round-trip promise. Identify exact versus
rounding conversions. State whether source literals and input text share a
grammar or intentionally differ. Decide how existing bit semantics remain
explicit. Update planning files only; no new numeric types or API spellings
are selected implicitly.

## Documentation and verification

The current [operator](../../../pascal/language/basics/operators.md),
[literal](../../../pascal/language/basics/number-literals.md), and text
references remain unchanged. The decision table must cover the reproduced
counterexamples, boundaries, mixed operands, and constant/runtime parity.
AP33.2/AP33.3 cannot start with unresolved cells that affect their behavior.
