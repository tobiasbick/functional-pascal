# Operators

Precedence, high to low:

1. Calls, member/index access, record `with` updates.
2. Unary `+`, `-`, `not`, `await`.
3. `*`, `/`, `div`, `mod`.
4. `+`, `-`.
5. `shl`, `shr`.
6. `<`, `<=`, `>`, `>=`, `is`.
7. `=`, `<>`.
8. A chain of `and`, `or`, or `xor`.
9. `??` (right associative).

Other binary operators associate left. Comparison/equality operators do not
chain. Parenthesize expressions combining `await` and coalescing when their
grouping would be unclear: `(await Load()) ?? 'Fallback'`.

Use one operator kind per logical chain. Write `(A and B) or C` or
`A and (B or C)` to mix kinds. The parser diagnoses the `or` in both
`A and B or C` and `A and (B) or C`. Each operand of `??` has its own
logical chain.

- `and`/`or` on `bool` short-circuit. `xor` is eager.
- On integral values, `and`, `or`, `xor`, `not`, `shl`, and `shr` are bitwise,
  with CLR/C# numeric promotion and shift-count rules. Same-type enum bitwise
  operations retain the enum type. Shift counts use their low five bits for
  32-bit operands and low six bits for 64-bit operands, including negative
  counts. Native operands use the execution runtime's pointer width.
- With two integral operands, `/` converts each to `double` before dividing.
  This includes mixed signed and unsigned types, native integers, `char`,
  and subranges projected to their base type. `1 / 2` yields `0.5`,
  `1 / 0` yields positive infinity, and `0 / 0` yields NaN. If an operand
  is already `float`, `double`, or `decimal`, use ordinary promotion.
- With built-in numeric operands, `div` and `mod` take integral operands.
  Division truncates toward zero.
- Numeric widening and binary promotion follow C# rules. For example,
  `int + long` produces `long`, and `short + short` produces `int`.
  `decimal` does not implicitly mix with `float` or `double`.
- Integral arithmetic and narrowing conversions check overflow by default.
  `unchecked(Expression)` enables wrapping integral arithmetic/conversions only
  inside that expression. Division and decimal arithmetic retain CLR exceptions.
- Floating operations follow IEEE 754 and may produce infinity or NaN.
- `string + string`, `string + char`, and `char + string` concatenate.
  Arbitrary objects are not implicitly formatted into strings.
- `Value ?? Fallback` evaluates the fallback only if the nullable value is null.
  For `Nullable<T>`, the non-null result is the contained `T`.

Compile-time constant evaluation checks both 32-bit and 64-bit native-integer
widths. A required constant must produce the same value without an exception
at both widths; an architecture-dependent result is diagnosed. This applies to
optional defaults, scalar bounds and enum values, constant patterns, and
attribute arguments. Ordinary runtime expressions retain native arithmetic
at the execution runtime's pointer width.

`=` and `<>` use primitive equality or an applicable CLR equality operator.
Strings compare ordinal contents; arrays use reference identity. Generated
record/variant equality is described in [types](../types/README.md). Other reference types
without an applicable equality operator use reference identity, not an implicit
call to virtual `Equals`. Ordering requires an applicable primitive/operator rule.

## Imported CLR operators

Concrete CLR operands can supply public static operator methods with supported
signatures. Binary lookup considers both operand types and unary lookup uses
the operand type. Ordinary overload rules select the method and its declared
result type. For example, subtracting two DateTimeOffset values yields a
TimeSpan, while subtracting a TimeSpan from a DateTimeOffset yields a
DateTimeOffset. Ambiguity is an error.

The existing symbols map to CLR operator members: +, -, *, / and div, mod,
and/or/xor, shl/shr, unary +/-, not, and comparisons. Both / and div select
op_Division for imported operators. Not selects op_OnesComplement or, if
unavailable, op_LogicalNot. Imported operators evaluate operands once in written
order. For concrete operands, a bool left operand makes and/or short-circuit
and requires a bool right operand. Built-in numeric, Boolean, and string rules
apply to their own operand combinations.

Checked arithmetic selects an applicable op_CheckedAddition, op_CheckedSubtraction,
op_CheckedMultiply, op_CheckedDivision, or op_CheckedUnaryNegation when supplied
by the CLR type or interface contract, otherwise the ordinary operator. Unchecked
selects the ordinary operator. The CLR implementation determines overflow and
exception behavior. Supported value operators have
[lifted nullable forms](../types/nullable-types.md#lifted-value-operators).
See [operator acceptance](../../../../tests/coverage.md).

## Operators from generic constraints

For a type parameter, operators resolve from its imported CLR interface
constraints, including inherited contracts such as INumber<T>. Binary lookup
uses both operands' type parameters and normal overload applicability. It
retains the selected interface, implementing parameter, argument types, and
result type. Unknown type parameters do not undergo implicit numeric promotion.
Missing and ambiguous contracts are errors. Struct-constrained
nullable parameters support lifting when the selected operator has non-nullable
value parameter and result types. Generic equality does not establish that an
operand is non-null. FPAS source operator declarations and static interface
implementations remain outside this scope.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

## Source files

- `src/Fpas.Compiler/Binding/Bodies/`
  - [Binder.Operators.cs](../../../../src/Fpas.Compiler/Binding/Bodies/Binder.Operators.cs)
  - [Binder.Unary.cs](../../../../src/Fpas.Compiler/Binding/Bodies/Binder.Unary.cs)
- `tests/Fpas.Compiler.Tests/ControlFlow/`
  - [ComparisonOperatorTests.cs](../../../../tests/Fpas.Compiler.Tests/ControlFlow/ComparisonOperatorTests.cs)
  - [OperatorSpacingTests.cs](../../../../tests/Fpas.Compiler.Tests/ControlFlow/OperatorSpacingTests.cs)
- `tests/Fpas.Compiler.Tests/Metadata/`
  - [ShiftEmissionTests.cs](../../../../tests/Fpas.Compiler.Tests/Metadata/ShiftEmissionTests.cs)
- `tests/Fpas.IntegrationTests/Build/`
  - [MsBuildTests.Comparisons.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.Comparisons.cs)
  - [MsBuildTests.GenericMath.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.GenericMath.cs)
  - [MsBuildTests.ImportedOperators.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.ImportedOperators.cs)
- `tests/Fpas.IntegrationTests/Execution/`
  - [ComparisonExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/ComparisonExecutionTests.cs)
