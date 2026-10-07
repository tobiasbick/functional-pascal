# Local variables

Variables can be declared inline inside statement lists. Every binding starts
with its own `const` or `var` keyword. The same rule
applies in routines and nested scopes. Local type declarations are not permitted.
`const` bindings are immutable; `var` allows reassignment.
Prefer `const` for immutable bindings in application and library code.
See [Constants](constants.md) for runtime initialization and static classification.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_stmt` and `var_stmt` in statement position).

```pascal
function FullName(First: string; Last: string): string;
begin
  const Space: string := ' ';
  return First + Space + Last;
end function;
```

## See also

- [Variables](variables.md)
