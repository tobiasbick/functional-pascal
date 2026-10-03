# Variables

Variables are **immutable by default**. Use `mutable var` to allow reassignment.
Both forms work as top-level declarations and as inline statements in a body.
Repeat `var` or `mutable var` for each binding; declaration groups are invalid.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`var_block`, `var_stmt`, `mutable_var_block`, and `mutable_var_stmt`).

```pascal
var Name: string := 'Alice'; // Immutable — cannot be reassigned
mutable var Age: integer := 30;

```

Reassigning an immutable variable is a compile-time error:

```pascal
var X: integer := 10;

begin
  X := 20;  // Error: cannot assign to immutable variable 'X'
end;
```

Mutable variables can be reassigned freely:

```pascal
mutable var Count: integer := 0;

begin
  Count := Count + 1;  // Valid mutable assignment
end;
```

Inline mutable variables use the same syntax:

```pascal
begin
mutable var Count: integer := 0;
Count := Count + 1;
end;
```

## See also

- [Local variables](local-variables.md)
- [Records — immutability](../types/records.md#immutability)
