# Type aliases

Create semantic names for existing types.

An alias retains the original type identity. Record aliases also retain field
defaults and visibility, including across unit reexports and inside collection
aliases. Required fields stay required; an alias cannot expose a private field.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_def`, `type_expr` — alias form).

```pascal
type UserId = integer;

type UserName = string;

type Callback = function(Value: integer): boolean;

```

Aliases to enum types retain qualified variant access. This is useful when a public API exposes an
enum owned by an internal unit:

```pascal
uses Std.Console as Console;

type PaletteColor = Console.Color;

var Value: PaletteColor := PaletteColor.Green;
```

## See also

- [Function types](../functions/function-types.md)
