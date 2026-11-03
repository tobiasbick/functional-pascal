# Operators

Precedence, high to low:

1. Calls, member/index access, record `with` updates.
2. Unary `+`, `-`, `not`, `await`.
3. `*`, `/`, `div`, `mod`.
4. `+`, `-`.
5. `shl`, `shr`.
6. `<`, `<=`, `>`, `>=`, `is`.
7. `=`, `<>`.
8. `and`, then `xor`, then `or`.
9. `??` (right associative).

Other binary operators associate left. Comparison/equality operators do not
chain. Parenthesize expressions combining `await` and coalescing when their
grouping would be unclear: `(await Load()) ?? 'Fallback'`.

- `and`/`or` on `bool` short-circuit. `xor` is eager.
- On integral values, `and`, `or`, `xor`, `not`, `shl`, and `shr` are bitwise,
  with CLR/C# numeric promotion and shift-count rules. Same-type enum bitwise
  operations retain the enum type.
- `/` accepts promoted `float`, `double`, or `decimal` operands. Two integral
  operands require `div` or an explicit floating conversion.
- `div` and `mod` accept integral operands; division truncates toward zero.
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

`=` and `<>` use primitive equality or an applicable CLR equality operator.
Strings compare ordinal contents; arrays use reference identity. Generated
record/union equality is described in [types](../types/README.md). Other reference types
without an applicable equality operator use reference identity, not an implicit
call to virtual `Equals`. Ordering requires an applicable primitive/operator rule.
Applicable CLR comparison operators are resolved from both operand types using
ordinary overload rules, including for value types such as DateTime and TimeSpan.
Ambiguous operators are errors. Supported value operators have
[lifted nullable forms](../types/nullable-types.md#lifted-value-operators);
source operator declarations remain unsupported. See
[comparisons and acceptance](../../../../tests/coverage.md).

## Operators from generic constraints

For a type parameter, operators resolve from its imported CLR interface
constraints, including inherited contracts such as INumber<T>. Binary lookup
uses both operands' type parameters and normal overload applicability; it retains
the selected interface, implementing parameter, argument types, and result type.
There is no implicit numeric promotion of an unknown T. Missing and ambiguous
contracts are errors. Struct-constrained nullable parameters support lifting when
the selected operator has non-nullable value parameter and result types.

The existing symbols map to CLR operator members: +, -, *, / and div, mod,
and/or/xor, shl/shr, unary +/-, not, and comparisons. Both / and div select
op_Division when used through a constraint; the CLR contract supplies its result
and division behavior. Not selects op_OnesComplement or, if unavailable,
op_LogicalNot. Constrained and/or calls are eager; bool and/or retain their
ordinary short-circuit behavior. Generic equality does not establish that an
operand is non-null.

Checked arithmetic selects an applicable op_CheckedAddition, op_CheckedSubtraction,
op_CheckedMultiply, op_CheckedDivision, or op_CheckedUnaryNegation when supplied
by the contract, otherwise the ordinary operator. Unchecked selects the ordinary
operator. The implementation, including any CLR default body, determines its
overflow and exception behavior. This consumes imported contracts; it does not
declare FPAS operators or static interface implementations on source records.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
