# Number literals

Integers support decimal and hexadecimal notation. Underscores are allowed as visual separators between digits (exactly one `_` between digit groups — `1__2` is invalid):

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`integer_literal`, `real_literal`).

```pascal id=example-1 fragment context=body
const A: integer := 1_000_000; // One million with digit separators
const B: integer := $FF; // Hexadecimal notation for 255
const C: integer := $FFFF; // Hexadecimal notation for 65535
```

Real literals require digits on both sides of the decimal point. Scientific notation is supported:

```pascal id=example-2 fragment context=body
const X: real := 3.14;
const Y: real := 1.5e10; // Scientific notation for 15000000000.0
const Z: real := 3.0e-4; // Scientific notation for 0.0003
const W: real := 0.5; // Leading zero is required
```

`.5` and `5.` are **not** valid — always write `0.5` or `5.0`. Integer literals must fit in a signed 64-bit range (`9223372036854775807` max). Real literals that overflow to infinity **or underflow a non-zero mantissa to zero** (for example `1.0e-9999`) are rejected as out of range; a zero mantissa such as `0.0e-9999` is fine.

Negative numbers normally use unary minus: `-42` is `-(42)`. In an expression,
the direct forms `-9223372036854775808` and `-$8000000000000000` represent the
minimum integer. Their positive magnitudes cannot be used as integer values.
The minimum form is recognized immediately after `-`; a parenthesized positive
magnitude such as `-(9223372036854775808)` remains invalid.

```pascal id=example-3 fragment context=body
const Minimum: integer := -9223372036854775808;
const HexMinimum: integer := -$8000000000000000;
const Maximum: integer := 9223372036854775807;
```

Hexadecimal literals denote mathematical magnitudes, not unsigned bit patterns
reinterpreted as signed integers. `$FFFFFFFFFFFFFFFF` is out of range.

`Std.Text.Parse.Int` accepts the same decimal and `$` hex
bodies with an optional sign and surrounding whitespace. Their real counterparts
accept the same fractional/exponent forms and reject nonfinite values and nonzero
underflow to zero. Representable subnormal reals remain valid. See
[parsing](../../std/text/parse.md) and [conversion](../../std/text/format.md).

Identifiers use ASCII letters, digits, and `_` only (see the grammar `identifier` rule). A UTF-8 BOM
(`U+FEFF`) is ignored only when it is the first scalar in a source file. A later `U+FEFF` is an
unexpected character rather than invisible token-separating trivia.

## See also

- [Primitive types](primitive-types.md)
- [Operators](operators.md)
