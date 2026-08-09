# Comments and declaration documentation

Functional Pascal has one comment syntax: `//`. A comment starts with `//` and ends at `LF`,
`CRLF`, bare `CR`, or the end of the file.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (Part 1 — comments).

```pascal
// This is a comment.
var Count: integer := 1; // This is also a comment.
```

For multiple comment lines, prefix every line with `//`:

```pascal
// The next value is displayed in the status line.
// It is measured in seconds.
var ElapsedSeconds: integer := 0;
```

`{...}` and `(*...*)` are not valid comment syntax. The lexer reports `F0013` and suggests the
valid `//` form. A sequence starting with `{$` remains an invalid compiler-directive sequence and
reports `F0010`.

## Markdown documentation

A contiguous block of standalone `//` lines immediately before a declaration is that
declaration's Markdown documentation. There must be no blank source line between the comment block
and the declaration.

```pascal
// Returns the larger value.
//
// - `Left`: first candidate
// - `Right`: second candidate
function Max(Left: integer; Right: integer): integer;
begin
  if Left >= Right then
    return Left
  else
    return Right
end;
```

Tooling removes the indentation, the first `//`, and at most one following ASCII space from each
documentation line. The remaining text is joined with `\n` and rendered as Markdown. A line that
contains only `//` becomes a blank Markdown line.

A blank source line detaches a comment from the following declaration:

```pascal
// This comment is not declaration documentation.

var Count: integer := 1;
```

End-of-line comments are never declaration documentation. `fpas fmt` preserves both attachment and
detachment. VS Code completion and hover display attached Markdown; hover first shows the resolved
declaration signature. Go to Definition and Go to Type Definition continue to navigate code symbols.
Markdown text inside a comment does not create navigable code symbols.

## See also

- [Formatter style](../../tools/fmt-style.md)
- [Editor integration](../../tools/editor-integration.md)
