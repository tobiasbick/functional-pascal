# Arrays

Dynamic arrays that grow as needed (0-based indexing).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `array of`, array literals, indexing).

```pascal
const Numbers: array of (integer) := [1, 2, 3];
const Empty: array of (string) := [];

```

Operations:

```pascal
program Example;

uses Std.Arrays as Arrays;

const Numbers: array of (integer) := [1, 2, 3];
const Len: integer := Arrays.Length(Numbers); // 3
const First: integer := Numbers[0]; // 1
 var Items: array of (integer) := [1, 2];

begin
  Arrays.Push(var Items, 3); // [1, 2, 3]
end program;
```

Use `Std.Arrays` for `Map`, `Filter`, `Reduce`, and other helpers — see [`Std.Arrays`](../../std/collections/array/README.md).

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [For-in](../control-flow/for-in.md)
