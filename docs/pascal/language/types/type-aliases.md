# Type aliases

Create semantic names for existing types.

Aliases can refer to types declared later in the same unit or program. An
alias-only cycle is rejected; aliases to finite recursive records and enums
remain valid. See [type declaration order](declaration-order.md).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_def`, `type_expr` — alias form).

```pascal
type UserId = integer;

type UserName = string;

type Callback = function(Value: integer): boolean;
```

Aliases to enum types retain qualified variant access. This is useful when a public API exposes an
enum owned by an internal unit:

```pascal
type PaletteColor = Color;

var Value: PaletteColor := PaletteColor.Green;
```

## See also

- [Function types](../functions/function-types.md)
