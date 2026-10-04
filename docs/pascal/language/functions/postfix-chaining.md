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
| `(Arguments)` | Invoke the preceding callable value with explicit positional arguments |

Suffixes evaluate left to right. Each step receives the value and static type of
the previous step. The base expression and every argument are evaluated exactly
once in source order. When a generic call infers a concrete return type from its
arguments, later suffixes use that concrete type.

```pascal
const Green: integer := BuildPalette().Normal.Foreground.Green;
const First: string := LoadItems()[0];
const Mapped: integer := MakeCallbacks().Transform(3);
```

Qualified root calls such as `Math.Sqrt(4.0)` after `uses Std.Math as Math;` remain ordinary calls. Only
suffixes that follow a completed primary become postfix operations:

```pascal
CreateRecord().Value
MakeCallbacks().Transform(2)
CreateItems()[0]
(CreateRecord()).Value
MakeAdder(3)(5)
Callbacks[0](42)
(Callback)(42)
```

## Calls after an expression

`(Arguments)` calls the preceding function or procedure value. Callable record
fields also use their own declared signature when called as `.Field(Arguments)`;
no receiver argument is inserted. Ordinary routines receive all arguments
explicitly, for example `Transform(Value, Amount)`.

Procedures may appear only as the final call of a postfix chain used as
a statement. A procedure call produces no value:

```pascal
MakeHandlers().OnDone();
Callbacks[0]();
```

Every earlier step must still produce a value. A procedure cannot appear in the
middle of a chain, and a postfix statement cannot end in a field or index. A
final function call requires consumption by another expression or an explicit
`discard`, just like an ordinary function call.

```pascal
discard MakeAdder(3)(5);
```

## Indexing

Postfix `[Index]` follows the same rules as designator indexing:

- array: integer index → element type
- dictionary: key type → value type
- string: integer index → `string`

## Formatter

Short chains stay on one line. When a chain exceeds the 100-column limit, the
formatter breaks before each suffix and indents continuations by two spaces from
the expression base column. A call immediately following a field stays attached
to that field, as in `.Transform(2)`. See [`fmt-style.md`](../../tools/fmt-style.md).

## See also

- [Functions](README.md)
- [Parameters](parameters.md)
