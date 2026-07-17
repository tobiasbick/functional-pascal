# Expression postfix chaining

A primary expression can be followed by one or more postfix suffixes that operate
on its result. Suffixes bind as tightly as designator field and index access —
tighter than unary, multiplicative, additive, comparison, and record-update
expressions.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`primary_expr`,
`postfix_suffix`).

## Suffixes

| Suffix | Meaning |
|--------|---------|
| `.Field` | Record field access |
| `[Index]` | Array, dictionary, or string index |
| `.Method(Arguments)` | Instance method call on a record value |

Suffixes evaluate left to right. Each step receives the value and static type of
the previous step. The base expression and every argument are evaluated exactly
once in source order. When a generic call infers a concrete return type from its
arguments, later suffixes use that concrete type.

```pascal
var Green: integer := BuildPalette().ForRole(TuiStyleRole.Normal).Foreground.Green;
var First: string := LoadItems()[0];
var Scaled: integer := Num.Create(3).Scale(2).Next().Value;
```

Qualified root calls such as `Std.Math.Sqrt(4.0)` remain ordinary calls. Only
suffixes that follow a completed primary become postfix operations:

```pascal
Factory.Create().Value
Factory.Create().Transform(2).Value
Factory.Create()[0]
(Factory.Create()).Value
```

## Instance methods

`.Method(...)` on a value resolves an instance method. Static record functions
stay callable only through a type designator (`Point.Create(...)`), not through
a returned value.

Procedure methods cannot appear in an expression chain because they do not
return a value. Statement chaining (discarding a final result) is not part of
this feature.

## Indexing

Postfix `[Index]` follows the same rules as designator indexing:

- array: integer index → element type
- dictionary: key type → value type
- string: integer index → `string`

## Formatter

Short chains stay on one line. When a chain exceeds the 100-column limit, the
formatter breaks before each suffix and indents continuations by two spaces from
the expression base column. See [`fmt-style.md`](../../tools/fmt-style.md).

## See also

- [Functions](README.md)
- [Record methods](../types/record-methods.md)
- [Parameters](parameters.md)
