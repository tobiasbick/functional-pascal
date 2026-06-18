# Local variables

Variables can be declared inline inside `begin..end` blocks:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`var_decl` in statement position).

```pascal
function FullName(First: string; Last: string): string;
begin
  var Space: string := ' ';
  return First + Space + Last;
end;
```

## See also

- [Variables](variables.md)
