# Future: Record Extensions

> Planned. Adds default field values, update expressions, and recursive record types.

## Default Field Values

### Motivation

Records with many fields require specifying every field at construction. This is verbose when most fields share common defaults (e.g., colors, visibility flags, dimensions).

### Proposed Syntax

```pascal
type
  ViewState = record
    X: integer := 0;
    Y: integer := 0;
    Width: integer := 80;
    Height: integer := 25;
    Visible: boolean := true;
    FgColor: integer := LightGray;
    BgColor: integer := Black;
  end;
```

Construction can then omit fields that keep their defaults:

```pascal
var V: ViewState := record
  Width := 40;
  Height := 10;
end;
{ X=0, Y=0, Width=40, Height=10, Visible=true, FgColor=LightGray, BgColor=Black }
```

All fields without defaults remain mandatory at construction.

---

## Record Update Expression

### Motivation

Immutable records require constructing a full new record to change one field. A `with` expression creates a copy with selected fields overridden:

### Proposed Syntax

```pascal
var V2: ViewState := V with
  X := 10;
  Visible := false;
end;
```

This yields a new `ViewState` where `X` and `Visible` are replaced; all other fields are copied from `V`. The original `V` is unchanged.

For single-field updates, a compact form:

```pascal
var V3: ViewState := V with X := 20 end;
```

---

## Recursive Record Types

### Motivation

Currently, enums with associated data support recursive definitions (`List<T>`), but records cannot reference themselves. A record type like:

```pascal
type
  TreeNode = record
    Value: string;
    Children: array of TreeNode;   { recursive: contains own type }
  end;
```

requires the compiler to allow self-referential record fields — at least through indirection via `array of T`, `Option of T`, or (once available) `ref T`.

### Proposed Rules

A record field may reference its own type when wrapped in:
- `array of T` — a dynamic, heap-allocated collection
- `Option of T` — an optional value (either `Some` or `None`)
- `ref T` — a reference type (see [references.md](references.md))

Direct self-reference without wrapping remains illegal (infinite size):

```pascal
type
  Bad = record
    Next: Bad;             { ERROR — infinite size }
  end;

  Good = record
    Children: array of Good;    { OK — indirection through array }
    Parent: Option of Good;     { OK — indirection through Option }
  end;
```

---

## Docs and Specs to Extend

- [05-types.md](../pascal/05-types.md): default field values, `with` expression, recursive record rules
- [grammar.ebnf](../specs/grammar.ebnf): `RecordFieldDefault`, `WithExpr` productions
- Compiler: emit default values for omitted fields at record construction
- Sema: validate recursive record references are wrapped in indirection
