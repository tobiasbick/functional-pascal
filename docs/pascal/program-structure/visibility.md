# Visibility

All declarations in a unit are **public by default**. Use the `private` keyword to restrict a declaration to the unit that defines it.

Formal syntax: [`grammar.ebnf`](../../specs/grammar.ebnf) (`visibility` on
declarations and `record_member`).

| Annotation | Meaning |
|---|---|
| *(none)* | Public (default) — visible to importers |
| `public` | Public (explicit, optional) — same as default |
| `private` | Unit-internal — excluded from the unit export table |

`private` declarations are compiled and available within the unit. Importers reference public symbols by short or qualified name only.

```pascal
unit MyApp.Geometry;
uses Std.Math;

type
  Point = record
    X: real;
    Y: real;
  end;

function Distance(A: Point; B: Point): real;
begin
  return Sqrt(Square(B.X - A.X) + Square(B.Y - A.Y))
end;

private function Square(V: real): real;
begin
  return V * V
end;
```

`Point` and `Distance` are public. `Square` is private — only callable from within `MyApp.Geometry`.

The `private` and `public` keywords apply to `function`, `procedure`, `type`,
`const`, and `var` declarations in units. They also apply directly to individual
record fields, functions, and procedures declared in a unit. Record members are
public by default, and there are no visibility sections. See
[Records](../language/types/records.md#field-visibility) and
[Record methods](../language/types/record-methods.md#routine-visibility).

In `program` files, visibility modifiers are invalid, including inside record
types.

Project-level unit export lists (`[exports].units` on library projects) are documented in [Projects](projects.md#exports-section-library-projects-only).

## See also

- [Units](units.md)
- [Projects — exports](projects.md#exports-section-library-projects-only)
