# Operators

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

`integer` is signed 64-bit. Addition, subtraction, multiplication and unary
negation are checked, as are `div` and `mod`. Overflow produces runtime panic
F4012 in every build mode. Integer division truncates toward zero; remainder has
the dividend's sign. A zero divisor produces F4001 for `div` and F4002 for `mod`.
The minimum integer divided by `-1`, or used with `mod -1`, also produces F4012.
Integer failures reached by [static evaluation](constants.md) are compile-time
errors F2020. Optimization preserves runtime failures for non-static expressions.

`real` uses IEEE binary64 arithmetic. `/` produces a real even for integer
operands. Division by positive or negative zero can produce signed infinity;
`0.0 / 0.0` produces NaN. Real arithmetic may produce infinity or NaN without an
integer-domain panic. NaN compares unequal to every value, including itself;
ordered comparisons involving NaN return false. Positive and negative zero
compare equal. These rules also apply to constant evaluation.

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
and simple enums) plus compatible `Option` and `Result` values. Records and enums with associated
data compare structurally when every field compares: two records are equal when all their fields
are equal, and two enum values are equal when they have the same variant and equal fields. Both
operands must have compatible types. Arrays compare their elements in order.
Dictionaries with distinct keys compare the same key/value mapping, independently
of insertion
order. Equality checks every nested component, including Option and Result
payloads; resource handles, tasks and callables have no whole-value equality,
and neither does data containing them. Real components use scalar equality:
NaN differs from itself, and positive and negative zero compare equal. Ordering operators (`<`, `>`, `<=`, `>=`) never apply to records or
enums with data.

`in` returns `boolean`. It tests whether an array contains a value, whether a dictionary contains a key, or whether a string contains a substring (or a single-character string):

```pascal
uses Std.Console as Console;

Console.WriteLn(2 in [1, 2, 3]);
Console.WriteLn('Alice' in ['Alice': 30]);
Console.WriteLn('a' in 'pascal');
Console.WriteLn('asc' in 'pascal');
```

## Operator precedence and evaluation

From strongest to weakest:

| Level | Operators | Grouping |
|-------|-----------|----------|
| 1 | Call, indexing, field selection, `with ... end with` | Postfix |
| 2 | Unary `-`, `try` | Prefix |
| 3 | `*`, `/`, `div`, `mod` | Left associative |
| 4 | `+`, `-` | Left associative |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in` | One comparison |
| 6 | `not` | Boolean prefix |
| 7 | `and`, `or`, `xor` | Same-operator chains only |

`X > 0 and Y > 0` compares both values. `not X > 0` means `not (X > 0)`.
Use parentheses to mix logical operators: `(A and B) or C` or `A and (B or C)`.
`A and B or C` is rejected. Arithmetic and same-operator logical chains group
from the left; `A - B - C` means `(A - B) - C`.

Comparison chains such as `A < B < C` are rejected. Write `(A < B) and (B < C)`.
If the middle expression has side effects, evaluate it once into a local binding
and use that binding in both comparisons.

Operands evaluate from left to right. `and` skips its right operand when the left
is false; `or` skips it when the left is true. `xor` evaluates both operands once.
`not`, `and`, `or`, and `xor` accept only booleans. The compiler type-checks both sides,
including operands skipped at runtime. The evaluation rules also apply to
constant initializers and debugger expressions.

Integer bit operations use explicitly imported [Std.Bits](../../std/numeric/bits.md)
functions. `shl` and `shr` are ordinary identifiers; their former infix uses are
rejected with migration guidance.

## String indexing

Individual characters can be read by 0-based integer index using bracket notation. The result type is `string` (a single-character string).

```pascal
const S: string := 'Hello';
const C: string := S[0]; // 'H'
const L: string := S[4];

```

Accessing an out-of-bounds index is a **runtime error**. The index must be an `integer`; non-integer indices are a compile-time error.

String indices are read-only. An assignment such as `S[0] := 'h'` is a
compile-time error, including when the string is a record field or collection
element. To change text, assign a whole replacement string to a mutable binding,
field, or collection element, for example `S := 'hello'` or `Items[0] := 'hello'`.

```pascal
uses Std.Console as Console;
uses Std.Str as Str;

// iterate over characters
 var I: integer := 0;
while I < Str.Length(S) do
  begin
    Console.WriteLn(S[I]);
    I := I + 1;
  end;
end while;
```

## String concatenation

```pascal
const Full: string := ('Hello' + ' ') + 'World';

```

## See also

- [Record update](../types/record-update.md)
- [Error handling — `try`](../error-handling/try.md)
