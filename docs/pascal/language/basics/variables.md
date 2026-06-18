# Variables

Variables are **immutable by default**. Use `mutable var` to allow reassignment. This works both as a declaration block and as an inline statement inside a `begin..end` block.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`var_decl`, `mutable`).

```pascal
var
  Name: string := 'Alice';       { immutable — cannot be reassigned }

mutable var
  Age: integer := 30;            { mutable — can be reassigned }
```

Reassigning an immutable variable is a compile-time error:

```pascal
var
  X: integer := 10;

begin
  X := 20;  { ERROR: cannot assign to immutable variable 'X' }
end.
```

Mutable variables can be reassigned freely:

```pascal
mutable var
  Count: integer := 0;

begin
  Count := Count + 1;  { OK }
end.
```

Inline mutable variables use the same syntax:

```pascal
begin
  mutable var Count: integer := 0;
  Count := Count + 1
end.
```

## See also

- [Local variables](local-variables.md)
- [Records — immutability](../types/records.md#immutability)
