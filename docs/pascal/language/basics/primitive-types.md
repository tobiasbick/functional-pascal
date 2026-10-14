# Primitive types

| Type      | Description                  | Example              |
|-----------|------------------------------|----------------------|
| `integer` | 64-bit signed integer        | `42`, `-7`, `0`      |
| `real`    | 64-bit floating point        | `3.14`, `-0.5`       |
| `boolean` | Boolean                      | `true`, `false`      |
| `string`  | Immutable text sequence      | `'Hello'`, `'A'`     |

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (literals, `type_expr` built-ins).

Single-character text uses the same `string` type as longer text:

```pascal
const Letter: string := 'A';
const Word: string := 'Hello';
```

Strings use single quotes with doubled apostrophes for escaping: `'It''s Pascal'`.

Strings may span multiple lines:

```pascal
const Poem: string := 'Roses are red'#10'Violets are blue';
```

## Character codes

The `#N` form denotes one Unicode scalar by its decimal value: **0..1114111**,
excluding the surrogate range **55296..57343**. It produces a `string`, with the
same value as `string.Chr(N)` for valid N. Invalid scalars and oversized numeric
values are lexical errors (FP1005); no value is truncated or replaced.

Character codes use the [decimal digit-separator rules](number-literals.md):
one underscore may separate digits, as in `#8_364`. A sign or hexadecimal prefix
is not allowed. Adjacent quoted strings and character codes concatenate without
an operator; whitespace separates literals.

```pascal
const LineBreak: string := #13#10; // CR+LF
const Greeting: string := 'Hello'#13#10'World'; // Hello\r\nWorld
const Tab: string := #9; // Tab character
const Euro: string := #8_364; // €
const Smile: string := #128512; // 😀
const Label: string := 'Price: '#8364; // Price: €
```

Codes remain compile-time string literals and can be used in constant case
labels. Each denotes one scalar, including `#0` and supplementary-plane values.
String length and indices count scalars, with no normalization or grapheme
processing. The [runtime character factory](../types/string/format-chars.md#stringchrn-integer-string)
accepts the same scalar domain and reports invalid dynamic values at runtime.

## See also

- [Number literals](number-literals.md)
- [Operators — string indexing](operators.md#string-indexing)
