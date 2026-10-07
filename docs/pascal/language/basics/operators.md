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
and simple enums) plus compatible `Option` and `Result` values. Records and enums with associated
data compare structurally when every field compares: two records are equal when all their fields
are equal, and two enum values are equal when they have the same variant and equal fields. Both
operands must have the same type. Arrays, dictionaries, callables, tasks, and channels have no
whole-value equality, and neither does a record or enum that contains one; compare their relevant
fields or contents explicitly. Ordering operators (`<`, `>`, `<=`, `>=`) never apply to records or
enums with data.

`in` returns `boolean`. It tests whether an array contains a value, whether a dictionary contains a key, or whether a string contains a substring (or a single-character string):

```pascal
WriteLn(2 in [1, 2, 3]);
WriteLn('Alice' in ['Alice': 30]);
WriteLn('a' in 'pascal');
WriteLn('asc' in 'pascal');
```

## Operator precedence

From highest to lowest binding strength:

| Level | Operators | Grouping |
| ----- | --------- | -------- |
| 1 | Postfix call, index, field, `with ... end with` | |
| 2 | Unary `-`, `try` | Prefix |
| 3 | `*`, `/`, `div`, `mod` | Left to right |
| 4 | `+`, `-` | Left to right |
| 5 | `=`, `<>`, `<`, `>`, `<=`, `>=`, `in` | Non-associative |
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
expression. Parentheses override the table in programs and debugger expressions.

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

Import `uses Std.Bits;` and call the named integer functions:

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

```pascal
const Denominator: integer := 0;
const NonzeroQuotient: boolean := (Denominator <> 0) and (10 div Denominator > 0);
// NonzeroQuotient is false; division is skipped.
```

Boolean `xor` evaluates both operands. Calls to binary `Std.Bits` functions
always evaluate both arguments, including `BitAnd(0, Right)` and `BitOr(-1, Right)`.
Debugger watch expressions follow these same evaluation rules.

## String indexing

Individual characters can be read by 0-based integer index using bracket notation. The result type is `string` (a single-character string).

```pascal
const S: string := 'Hello';
const C: string := S[0]; // 'H'
const L: string := S[4]; // 'o'
```

Accessing an out-of-bounds index is a **runtime error**. The index must be an `integer`; non-integer indices are a compile-time error.

```pascal
// iterate over characters
var I: integer := 0;
while I < Std.Str.Length(S) do
  begin
    WriteLn(S[I]);
    I := I + 1;
  end;
end while;
```

## String concatenation

```pascal
const Full: string := ('Hello' + ' ') + 'World'; // 'Hello World'
```

## See also

- [Std.Bits — named integer bit operations](../../std/numeric/bits.md)
- [Record update](../types/record-update.md)
- [Error handling — `try`](../error-handling/try.md)
