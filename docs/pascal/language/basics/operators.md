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
Ambiguous operators are errors. This does not introduce lifted nullable-value
operators or source operator declarations. See
[comparisons and acceptance](../../../implementation/comparisons-and-acceptance.md).

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
