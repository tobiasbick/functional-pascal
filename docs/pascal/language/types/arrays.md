# Arrays

Dynamic arrays that grow as needed (0-based indexing).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `array of`, array literals, indexing).

```pascal
const Numbers: array of integer := [1, 2, 3];
const Empty: array of string := [];
```

Operations:

```pascal
const
  Len: integer := Length(Numbers);  // 3
  First: integer := Numbers[0];     // 1

var
  Items: array of integer := [1, 2];

begin
  Push(Items, 3);  // [1, 2, 3]
end.
```

Use `Std.Arrays` for `Map`, `Filter`, `Reduce`, and other helpers — see [`Std.Arrays`](../../std/collections/array/README.md).

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [For-in](../control-flow/for-in.md)
