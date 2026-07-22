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
and simple enums) plus compatible `Option` and `Result` values. Records, arrays, dictionaries, and
procedures have no whole-value equality; compare their relevant fields or contents explicitly.

`in` returns `boolean`. It tests whether an array contains a value, whether a dictionary contains a key, or whether a string contains a substring (or a single-character string):

```pascal
WriteLn(2 in [1, 2, 3]);
WriteLn('Alice' in ['Alice': 30]);
WriteLn('a' in 'pascal');
WriteLn('asc' in 'pascal')
```

## Operator precedence

From highest to lowest binding strength:

| Level | Operators |
| ----- | --------- |
| 1 | `not`, unary `-`, `try` |
| 2 | `*`, `/`, `div`, `mod`, `and`, `shl`, `shr` |
| 3 | `+`, `-`, `or`, `xor` |
| 4 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in` |

Record update (`expr with Field := Value; … end`) binds tighter than binary operators because it is postfix on the primary expression.

## Logical / bitwise

| Operator | Description                          | Example         |
|----------|--------------------------------------|------------------|
| `and`    | Logical AND / bitwise AND on integer | `A and B`       |
| `or`     | Logical OR / bitwise OR on integer   | `A or B`        |
| `not`    | Logical NOT / bitwise NOT on integer | `not A`         |
| `xor`    | Logical XOR / bitwise XOR on integer | `A xor B`       |
| `shl`    | Shift left (integer)                 | `A shl 2`       |
| `shr`    | Shift right (integer)                | `A shr 1`       |

## String indexing

Individual characters can be read by 0-based integer index using bracket notation. The result type is `string` (a single-character string).

```pascal
var
  S: string := 'Hello';
  C: string := S[0];   { 'H' }
  L: string := S[4];   { 'o' }
```

Accessing an out-of-bounds index is a **runtime error**. The index must be an `integer`; non-integer indices are a compile-time error.

```pascal
{ iterate over characters }
mutable var I: integer := 0;
while I < Std.Str.Length(S) do begin
  WriteLn(S[I]);
  I := I + 1
end
```

## String concatenation

```pascal
var
  Full: string := 'Hello' + ' ' + 'World';  { 'Hello World' }
```

## See also

- [Record update](../types/record-update.md)
- [Error handling — `try`](../error-handling/try.md)
