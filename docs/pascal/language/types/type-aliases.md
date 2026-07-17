# Type aliases

Create semantic names for existing types.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_decl` — alias form).

```pascal
type
  UserId = integer;
  UserName = string;
  Callback = function(Value: integer): boolean;
```

Aliases to enum types retain qualified variant access. This is useful when a public API exposes an
enum owned by an internal unit:

```pascal
type
  PaletteColor = Color;

var Value: PaletteColor := PaletteColor.Green;
```

## See also

- [Function types](../functions/function-types.md)
