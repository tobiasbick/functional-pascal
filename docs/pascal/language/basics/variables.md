# Variables

`var` declares a reassignable binding. Use `const` for an immutable binding.
Every binding repeats its own keyword, both at program or unit level and in
statement lists. Declaration groups are rejected with
[FP2015](../../tools/diagnostics.md#parser).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf)
(`var_declaration`, `var_stmt`, `const_declaration`, and `const_stmt`).

```pascal
const Name: string := 'Alice';
var Age: integer := 30;
Age := Age + 1;
```

Reassigning a `const`, or changing its stored record fields or collection elements,
is a compile-time error. The diagnostic suggests declaring a `var` when a
writable binding is needed. Assignments through a `var` can replace the value
or update its fields and elements; arrays, dictionaries, and records retain
value semantics. Shared handles retain their existing sharing rules.

Inline variables use the same syntax:

```pascal
begin
  var Count: integer := 0;
  Count := Count + 1;
end.
```

Each exported binding also repeats `public`; the modifier applies only to that
declaration. Initializers execute in source order, including consecutive
individual declarations. A `const` initializer may compute a value at runtime;
see [Constants](constants.md).

Loop variables are immutable inside each iteration. A captured local `var`
shares one mutable cell; a captured `const` copies its value. See
[Closures](../functions/closures.md).

## See also

- [Local variables](local-variables.md)
- [Records — immutability](../types/records.md#immutability)
