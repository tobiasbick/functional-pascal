# AP33.2: Arithmetic and literals

Package: [AP33](README.md). Status: complete.

## Scope

Numeric operators require matching integer, real, or declared Numeric type
parameters. Integer arithmetic is checked; `/` returns real, including on
integer and generic operands. Direct decimal/hex minimum integer expressions
are valid; positive magnitudes outside the signed range receive FP2021.
Negative division truncates toward zero, remainder follows the dividend, and
the minimum modulo -1 is zero. Real operands/results are finite, division by
either zero sign fails, and arithmetic permits gradual underflow. Round uses
ties to even; integer conversion results are checked. Distinct/subrange
identities and Std.Bits retain their existing contracts.

## Implementation ownership

| Concern | Owner |
| --- | --- |
| Checked arithmetic and numeric bounds | [`fpas-numeric`](../../../../crates/fpas-numeric/src/lib.rs), `integer.rs` and `real.rs` |
| Literal magnitude and direct signed minimum | [`lexer numbers`](../../../../crates/fpas-lexer/src/lexer/numbers.rs), parser [`primary`](../../../../crates/fpas-parser/src/parser/expr/primary.rs) and [`precedence`](../../../../crates/fpas-parser/src/parser/expr/precedence.rs) |
| Numeric compatibility | [`Sema operators`](../../../../crates/fpas-sema/src/check/expr/operators.rs) |
| Constant evaluation and folding | [`scalar constants`](../../../../crates/fpas-sema/src/check/decl/consts/scalar.rs), compiler [`constant folding`](../../../../crates/fpas-compiler/src/optimize/constant_folding.rs) |
| Typed/generic lowering | [`numeric lowering`](../../../../crates/fpas-compiler/src/lowering/expr/numeric.rs), IR result categories |
| Runtime and debugger operations | [`VM value operations`](../../../../crates/fpas-vm/src/vm/value_ops/mod.rs), typed execution; debugger uses the same value operations |
| Standard numeric operations | [`Std.Math`](../../../../crates/fpas-std/src/math/mod.rs), [`Std.Conv`](../../../../crates/fpas-std/src/conv/mod.rs) |
| Compiled artifact invalidation | [`bytecode version`](../../../../crates/fpas-bytecode/src/lib.rs) 20; existing unit/program version validation |

Compile-time evaluation does not persist wrapped or nonfinite results. Folding
keeps failing operations executable, including under discard, while preserving
short-circuit behavior. No Value layout change or new numeric type is required.

## Migration and documentation

Repository arithmetic consumers use explicit integer-to-real conversion.
Exported-default fixtures use the direct minimum literal. Program/bundle byte
goldens include version 20. Current [operators](../../../pascal/language/basics/operators.md),
[literals](../../../pascal/language/basics/number-literals.md),
[generics](../../../pascal/language/types/generics.md),
[conversion](../../../pascal/std/text/conv.md),
[math](../../../pascal/std/numeric/math.md), grammar comments,
[diagnostics](../../../pascal/tools/diagnostics.md), and
[compiled programs](../../../pascal/program-structure/compiled-programs.md)
describe the implemented behavior.

## Regression coverage

[`Compiler numeric contracts`](../../../../crates/fpas-compiler/src/tests/numeric_contracts.rs)
exercise literal, constant, local-variable and generic paths, integer/real
overflow, rounding, signed zero after arithmetic underflow, minimum values,
negative division/remainders and unselected operations.
[`Sema numeric contracts`](../../../../crates/fpas-sema/src/tests/expr/numeric.rs)
reject mixtures at operators, assignments, calls, fields and generic inference.
[`Parser expressions`](../../../../crates/fpas-parser/src/tests/expr/standalone.rs)
check oversized positive magnitudes and parenthesized minimum forms.
VM tests cover mixed raw numeric instructions and nonfinite ordering.
[`Exported defaults`](../../../../crates/fpas-build/tests/record_construction/mod.rs)
retain signed boundaries through compiled-unit reuse. Existing suites cover
division by zero, domain types, subranges, bit operations and debugger evaluation.

## Open follow-ups

None within this work package's scope.
