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

Use `Std.Array` for `Map`, `Filter`, `Reduce`, and other helpers — see [`Std.Array`](../../std/array.md).

## See also

- [Basics — arrays intro](../../02-basics.md#arrays)
- [Control flow — for-in](../../03-control-flow.md#for-in-array-iteration)
