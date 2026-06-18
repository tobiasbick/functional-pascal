# Type aliases

Create semantic names for existing types.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_decl` — alias form).

```pascal
type
  UserId = integer;
  UserName = string;
  Callback = function(Value: integer): boolean;
```

## See also

- [Functions — function types](../../04-functions.md#function-types)
