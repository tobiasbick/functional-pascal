# Local variables

Variables can be declared inline inside statement lists. Every binding starts
with its own `var` or complete `mutable var` prefix. The same rule applies in
routines and nested scopes; local type and constant declarations are not
permitted.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`var_stmt` and `mutable_var_stmt` in statement position).

```pascal
function FullName(First: string; Last: string): string;
begin
  var Space: string := ' ';
  return First + Space + Last;
end function;
```

## See also

- [Variables](variables.md)
