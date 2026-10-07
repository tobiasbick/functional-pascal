# Number literals

Integers support decimal and hexadecimal notation. Underscores are allowed as visual separators between digits (exactly one `_` between digit groups — `1__2` is invalid):

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`integer_literal`, `real_literal`).

```pascal
var A: integer := 1000000; // One million
var B: integer := 255; // 255 hexadecimal
var C: integer := 65535; // 65535 hexadecimal
```

Real literals require digits on both sides of the decimal point. Scientific notation is supported:

```pascal
var X: real := 3.14;
var Y: real := 15000000000.0; // Scientific notation
var Z: real := 0.0003; // 0.0003
var W: real := 0.5; // Leading zero is required
```

`.5` and `5.` are **not** valid — always write `0.5` or `5.0`. Integer literals must fit in a signed 64-bit range (`9223372036854775807` max). Real literals that overflow to infinity **or underflow a non-zero mantissa to zero** (for example `1.0e-9999`) are rejected as out of range; a zero mantissa such as `0.0e-9999` is fine.

Negative numbers are parsed as unary minus + literal: `-42` is `-(42)`.

Identifiers use ASCII letters, digits, and `_` only (see the grammar `identifier` rule). A UTF-8 BOM
(`U+FEFF`) is ignored only when it is the first scalar in a source file. A later `U+FEFF` is an
unexpected character rather than invisible token-separating trivia.

## See also

- [Primitive types](primitive-types.md)
- [Operators](operators.md)
