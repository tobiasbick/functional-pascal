# Arrays

Dynamic arrays that grow as needed (0-based indexing).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `array of`, array literals, indexing).

```pascal
var
  Numbers: array of integer := [1, 2, 3];
  Empty: array of string := [];
```

Operations:

```pascal
var
  Len: integer := Length(Numbers);  { 3 }
  First: integer := Numbers[0];              { 1 }

mutable var
  Items: array of integer := [1, 2];

begin
  Push(Items, 3);  { [1, 2, 3] }
end.
```

Use `Std.Array` for `Map`, `Filter`, `Reduce`, and other helpers — see [`Std.Array`](../../std/collections/array.md).

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [For-in](../control-flow/for-in.md)
