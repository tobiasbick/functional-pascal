# Visibility

Declarations in a unit are **private by default**. Write `public` directly
before a declaration that importing units may use. FPAS has no explicit
`private` keyword and no visibility sections.

Formal syntax: [`grammar.ebnf`](../../specs/grammar.ebnf) (`public_modifier` on
declarations and record members).

| Modifier | Meaning |
|---|---|
| *(none)* | Private — usable only within the declaring unit |
| `public` | Public — exported to importing units |

```pascal
unit MyApp.Geometry;

uses Std.Math;

public type
  Point = record
    public X: real;
    public Y: real;
  end;

function Square(V: real): real;
begin
  return V * V
end;

public function Distance(A: Point; B: Point): real;
begin
  return Sqrt(Square(B.X - A.X) + Square(B.Y - A.Y))
end;
```

`Point` and `Distance` are public. `Square` is private because it has no
`public` modifier.

The modifier applies to `function`, `procedure`, `type`, `const`, `var`, and
`mutable var` declarations in units. On records declared in units it also
applies directly to individual fields, functions, procedures, properties, and
events. Every such record member is private unless it is declared `public`.
See [Records](../language/types/records.md#field-visibility),
[Record methods](../language/types/record-methods.md#routine-visibility),
[Record properties](../language/types/record-properties.md#visibility), and
[Record events](../language/types/record-events.md#visibility).

The `public` modifier is invalid in `program` files, including inside record
types. The word `private` is an ordinary identifier.

Project-level unit export lists (`[exports].units` on library projects) are
documented in [Projects](projects.md#exports-section-library-projects-only).

## See also

- [Units](units.md)
- [Projects — exports](projects.md#exports-section-library-projects-only)
