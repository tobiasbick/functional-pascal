# Lexical rules and literals

Sources are UTF-8 `.fpas` files. LF, CRLF, and bare CR terminate lines; an initial
UTF-8 BOM is allowed. Whitespace between complete language elements is otherwise
insignificant; identifiers, literals and compound operators keep their spelling.
Comments start with
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

Generic closing `>` delimiters do not require surrounding whitespace:
`type Box<T>=record Value:T;end record;` is valid. Parsing identifies the generic
closing delimiter and the declaration's `=` separately. Compound comparison
operators retain their spelling: `X>=1` is valid; `X > = 1` is invalid. The
[formatter](../../tools/fmt-style.md) produces `Box<T> = record`, `Value: T;`
and `X >= 1`.

Spaces, tabs, Unicode whitespace, line endings and line comments can separate
language elements. For example, `Value : T ;`, `Call ( Value )` and
`Box< Box<int> >` are valid. Separate adjacent words such as `end if`; `endif`
is one identifier. `notValue` is also one identifier, distinct from `not Value`.
Comments cannot split a compound operator, numeric literal or escaped name.
A separator is necessary when adjacent elements would otherwise merge. Keep a
space between division `/` and a following `//` comment, as in `/ // comment`.
Without that space, `/// comment` starts a comment at the first slash and contains
no division operator.

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
