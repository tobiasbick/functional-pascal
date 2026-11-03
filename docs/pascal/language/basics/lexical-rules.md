# Lexical rules and literals

Sources are UTF-8 `.fpas` files. LF, CRLF, and bare CR terminate lines; an initial
UTF-8 BOM is allowed. Whitespace is otherwise insignificant. Comments start with
`//` and end at the next line ending or EOF. Only line comments are supported;
brace, parenthesis-star, and slash-star forms are not comments. Consecutive
standalone comments immediately before a declaration supply its Markdown
documentation; a blank source line detaches them. See
[source comments and API documentation](comments.md) for attachment, structured
parameter/return descriptions, and .NET XML export.

Keywords and identifiers are **case-sensitive**. Keywords are lowercase;
`Count` and `count` are different identifiers. Identifiers use ASCII letters,
digits, and `_`, with no leading digit. `_` alone is a pattern wildcard, not a
value binding. Prefix a keyword with `@` to use it as an identifier, including
in an imported CLR name: `Value.@event`. The `@` is not part of its metadata name.

Primitive aliases are predefined type names, not keywords. They cannot be
redeclared in a type namespace. See [types](../types/README.md).

### Literals

| Form | Meaning |
| --- | --- |
| `42`, `1_000`, `$FF` | Integral literal; default type `int` |
| `42L`, `42U`, `42UL` | `long`, `uint`, `ulong` |
| `1.5`, `1e3`, `1.5D` | `double` |
| `1.5F`, `1.5M` | `float`, `decimal` |
| `'Hello'`, `'It''s ready'` | `string`; doubled apostrophe escapes an apostrophe |
| `#65`, `#10` | `char`, one UTF-16 code unit, range 0..65535 |
| `true`, `false` | `bool` |
| `null` | Null literal for nullable CLR types |

Numeric suffixes accept either case. `_` may occur singly between digits.
Hexadecimal literals may have integral suffixes. Negative numbers use unary `-`;
a directly negated literal may represent the minimum signed value.
An `int` constant may be contextually converted to a smaller integral type,
an unsigned integral type, or a native integral type when representable;
a representable `long` constant may also convert to `ulong`. Char requires a
character literal or an explicit cast. Otherwise, an unsuffixed integral literal must fit `int`;
use an explicit suffix for larger values. Floating literals must fit their
chosen representation; use CLR APIs to obtain infinity or NaN.

Strings do not span source lines. Use concatenation and character literals:
`'First' + #10 + 'Second'`. There is no adjacent-literal concatenation. A `char`
may be a surrogate code unit; it is not a Unicode scalar. Use `System.Text.Rune`
when scalar processing is needed.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
