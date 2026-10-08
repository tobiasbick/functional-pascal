# Arrays

Dynamic arrays that grow as needed (0-based indexing).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `array of`, array literals, indexing).

```pascal
const Numbers: array of integer := [1, 2, 3];
const Empty: array of string := [];
```

Operations:

```pascal
program ArrayOperations;


const Numbers: array of integer := [1, 2, 3];
const Len: integer := Numbers.Length();  // 3
const First: integer := Numbers[0];     // 1
var Items: array of integer := [1, 2];

begin
  Items.Push(3);  // [1, 2, 3]
end.
```

Arrays provide `Map`, `Filter`, `Reduce`, `Length`, `IsEmpty`, and other operations
through dot calls without imports. `array.Fill(Value, Count)` constructs values — see [`Array operations`](array/README.md).

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [For-in](../control-flow/for-in.md)
