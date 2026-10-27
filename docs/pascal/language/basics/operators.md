# Operators

The code fragments use `uses Std.Conv as Conv, Std.Console as Console;`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (expression precedence).

## Arithmetic

| Operator | Description      | Example     |
|----------|------------------|-------------|
| `+`      | Addition         | `A + B`     |
| `-`      | Subtraction      | `A - B`     |
| `*`      | Multiplication   | `A * B`     |
| `/`      | Real division    | `A / B`     |
| `div`    | Integer division | `A div B`   |
| `mod`    | Modulo           | `A mod B`   |

[Subrange](../types/subranges.md) operands use their integer values for arithmetic
and comparisons. Integer arithmetic produces `integer`, and `/` produces `real`.
Logical operators retain their Boolean-only rules.

Both numeric operands must have the same type. FPAS does not implicitly mix
`integer` and `real` in arithmetic, equality, ordering, assignment, return
values, or call arguments. Import `Std.Conv` and convert explicitly:

```pascal id=example-1 fragment context=body-conv
const Sum: real := Conv.IntToReal(1) + 0.5;
const Before: boolean := Conv.IntToReal(1) < 1.5;
const Half: real := 1 / 2;
```

`/` is real division: two integer operands also produce a real result. Use
`div` for an integer quotient. Generic `Numeric` operands must use the same
type parameter; `/` returns real even in a generic body. See
[generics](../types/generics.md#built-in-constraints).

### Integer bounds and division

Integers are signed 64-bit values. `+`, `-`, `*`, and unary `-` reject a result
outside `-9223372036854775808..9223372036854775807` with a numeric-domain
diagnostic. Ordinary arithmetic does not wrap. Negating the minimum integer
and dividing it by `-1` overflow. Bit operations retain their separate
[Std.Bits](../../std/numeric/bits.md) rules.

`div` truncates toward zero. `mod` has the dividend's sign, and its absolute
value is smaller than the divisor's absolute value:

| Expression | Value |
| --- | --- |
| `-7 div 3` | `-2` |
| `-7 mod 3` | `-1` |
| `7 mod (-3)` | `1` |
| `-9223372036854775808 mod (-1)` | `0` |

Both integer operators reject a zero divisor. The minimum modulo `-1` is zero
because that remainder is representable even though the quotient is not.

### Real bounds

Real arithmetic uses binary64 rounding and gradual underflow. A computation
may produce a subnormal value or signed zero, but NaN and infinity are rejected
as operands or results. `/` rejects both `+0.0` and `-0.0` as divisors. Real
equality treats both signs of zero as equal; formatting preserves the sign.
Nonzero [literal](number-literals.md) or numeric input text that rounds to zero
is out of range; the arithmetic underflow rule does not change that input rule.

Constant evaluation, optimization, execution, and debugger evaluation follow
these same rules. A failing operation is not optimized away, including under
`discard`; an unselected branch or skipped short-circuit operand is not executed.

## Comparison

| Operator | Description       | Example    |
|----------|-------------------|------------|
| `=`      | Equal             | `A = B`    |
| `<>`     | Not equal         | `A <> B`   |
| `<`      | Less than         | `A < B`    |
| `>`      | Greater than      | `A > B`    |
| `<=`     | Less or equal     | `A <= B`   |
| `>=`     | Greater or equal  | `A >= B`   |
| `in`     | Membership        | `A in B`   |

Equality (`=` and `<>`) accepts compatible scalar values (`integer`, `real`, `boolean`, `string`,
simple enums, and [distinct types](../types/distinct-types.md)). `Option` and `Result` values,
records, and enums with associated data compare structurally when every payload or field compares:
two records are equal when all their fields are equal, two enum values are equal when they have
the same variant and equal fields, and `Some(A) = Some(B)` compares `A` with `B`. Both operands
of an aggregate comparison must have the same type. Arrays, dictionaries, callables, tasks, and channels have no whole-value
equality, and neither does an `Option`, `Result`, record, or enum that contains one, such as
`Option of array of integer`; compare their relevant fields or contents explicitly. Ordering operators (`<`, `>`, `<=`, `>=`) never apply to records or
enums with data.

Generic `=` and `<>` require `T: Equatable` or a stronger Comparable/Numeric
guarantee. Ordering requires Comparable or Numeric. Equatable follows these same
field/payload rules and does not add whole-array or dictionary equality.

`in` returns `boolean`. It tests whether an array contains a value, whether a dictionary contains a key, or whether a string contains a substring (or a single-character string):

```pascal id=example-2 fragment context=body-console
Console.WriteLn(2 in [1, 2, 3]);
Console.WriteLn('Alice' in ['Alice': 30]);
Console.WriteLn('a' in 'pascal');
Console.WriteLn('asc' in 'pascal');
```

With a [subrange type](../types/subranges.md#membership) on the right, `in`
tests its inclusive integer bounds: `0 in Percent` and `100 in Percent` are
`true` for `type Percent = 0..100;`, while `-1 in Percent` and `101 in Percent`
are `false`. The left operand must be an integer or subrange value; the right
operand names the type or a type alias, rather than a subrange value. The value
is evaluated once, and the test performs no conversion. `Input in Percent and
Input > 0` uses the existing comparison precedence and Boolean short-circuit
rules. A successful test still requires explicit `Percent(Input)` conversion
when a subrange value is needed.

## Operator precedence

From highest to lowest binding strength:

| Level | Operators | Grouping |
| ----- | --------- | -------- |
| 1 | Postfix call, index, field, `with ... end with` | |
| 2 | Unary `-`, `try` | Prefix |
| 3 | `*`, `/`, `div`, `mod` | Left to right |
| 4 | `+`, `-` | Left to right |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in`, `is` | Non-associative |
| 6 | `not` | Prefix |
| 7 | `and`, `or`, `xor` | Same-operator chains only, left to right |

Comparisons bind more tightly than `not`. Thus `not Count > 0` means
`not (Count > 0)`, and `not Done and Ready` means `(not Done) and Ready`.
Repeated negation is allowed: `not not Ready` means `not (not Ready)`.
To use a negated value inside a comparison or arithmetic expression, put it
in parentheses: `(not Done) = Ready`. Unary `-` and `try` keep their higher
priority: `not try GetFlag()` negates the unwrapped result.

Chains of one logical operator are valid, such as `A and B and C` or
`A xor B xor C`. Different logical operators require explicit parentheses:
`A and B or C` is a syntax error; write `(A and B) or C` or `A and (B or C)`
to select the intended grouping. There is no implicit priority between
`and`, `or`, and `xor`.

Comparison chains such as `A < B < C` are syntax errors. Write
`A < B and B < C` instead. Each comparison evaluates its own operands, so
a call used for `B` may run twice; store its result first when one call is
intended. Parentheses permit a comparison result as an operand, for example
`(A < B) = Expected`.

Record update (`expr with Field := Value; … end with`) is postfix on the primary
expression. An [`if` expression](../control-flow/if-then-else.md#if-expressions)
is a primary operand closed by `end if`, so `1 + if Ready then 2 else 3 end if`
needs no parentheses. Parentheses override the table in programs and debugger
expressions.

## Logical operators

| Operator | Description | Example |
|----------|-------------|---------|
| `and` | Logical AND | `Ready and Valid` |
| `or` | Logical OR | `Ready or Valid` |
| `not` | Logical NOT | `not Ready` |
| `xor` | Logical XOR | `Ready xor Valid` |

All operands and results are `boolean`. There is no implicit conversion from
numbers to booleans: compare explicitly, for example `Count > 0 and Ready`.
This also applies to an operand skipped by short-circuit evaluation;
`false and 1` is a type error.

## Integer bit operations

Import `uses Std.Bits as Bits;` and call the named integer functions:

| Operation | Function |
|-----------|----------|
| Bitwise AND | `BitAnd(Left, Right)` |
| Bitwise OR | `BitOr(Left, Right)` |
| Bitwise XOR | `BitXor(Left, Right)` |
| Bitwise complement | `BitNot(Value)` |
| Shift left | `ShiftLeft(Value, Count)` |
| Arithmetic shift right | `ShiftRight(Value, Count)` |

The functions operate on signed 64-bit integers. Shift counts must be `0..63`;
see [Std.Bits](../../std/numeric/bits.md) for results and runtime errors.
An integer operand of a logical operator receives a type error with a matching
`Std.Bits` hint. Both operands of a binary logical operator must be boolean;
a mixed boolean/integer expression receives a Boolean operand hint.

`shl` and `shr` are ordinary identifiers, so they may name variables, functions,
or record members. Infix uses such as `Value shl Count` are syntax errors with
a `Std.Bits.ShiftLeft(Value, Count)` hint; `shr` points to `ShiftRight`.

## Evaluation order

Binary operands are evaluated from left to right. Boolean `and` and `or` use
short-circuit evaluation: `false and Right` returns `false` without evaluating
`Right`; `true or Right` returns `true` without evaluating `Right`. Otherwise,
the right operand is evaluated and determines the result. Each needed operand
is evaluated exactly once.

An operand that is skipped performs no calls or other side effects, raises no
runtime errors, and does not propagate an error or `None` through `try`. It must
still be syntactically valid and pass normal compile-time type checking.

```pascal id=example-3 fragment context=body
const Denominator: integer := 0;
const NonzeroQuotient: boolean := (Denominator <> 0) and (10 div Denominator > 0);
// NonzeroQuotient is false; division is skipped.
```

Boolean `xor` evaluates both operands. Calls to binary `Std.Bits` functions
always evaluate both arguments, including `BitAnd(0, Right)` and `BitOr(-1, Right)`.
Debugger watch expressions follow these same evaluation rules.

## String indexing

Individual characters can be read by 0-based integer index using bracket notation. The result type is `string` (a single-character string).

```pascal id=example-4 fragment context=body
const S: string := 'Hello';
const C: string := S[0]; // 'H'
const L: string := S[4]; // 'o'
```

Accessing an out-of-bounds index is a **runtime error**. The index must be an `integer`; non-integer indices are a compile-time error.

```pascal id=example-5 fragment context=scalar-loop
// iterate over characters
var I: integer := 0;
while I < S.Length() do
  begin
    Console.WriteLn(S[I]);
    I := I + 1;
  end;
end while;
```

## String concatenation

```pascal id=example-6 fragment context=body
const Full: string := 'Hello' + ' ' + 'World'; // 'Hello World'
```

## See also

- [Std.Bits — named integer bit operations](../../std/numeric/bits.md)
- [Record update](../types/record-update.md)
- [Error handling — `try`](../error-handling/try.md)
