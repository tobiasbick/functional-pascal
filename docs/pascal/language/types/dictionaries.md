# Dictionaries

`dict of K to V` stores key-value pairs. Keys keep insertion order when iterated with `for-in`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `dict of`, dict literals, indexing).

```pascal
var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
var Empty: dict of string to integer := [:];

var AliceAge: integer := Ages['Alice'];
```

Dictionary writes require a mutable binding:

```pascal
mutable var
  Counts: dict of string to integer := ['A': 1];

begin
  Counts['A'] := 2;
  Counts['B'] := 3
end.
```

Use `Std.Dict` for helpers such as `Length`, `ContainsKey`, `Get`, `Keys`, `Values`, and `Remove` — see [`Std.Dict`](../../std/collections/dict.md).

## See also

- [For-in](../control-flow/for-in.md)
