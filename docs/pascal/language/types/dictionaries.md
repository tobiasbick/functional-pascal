# Dictionaries

`dict of K to V` stores key-value pairs. Keys keep insertion order when iterated with `for-in`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `dict of`, dict literals, indexing).

```pascal
const Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];
const Empty: dict of string to integer := [:];

const AliceAge: integer := Ages['Alice'];
```

Keys are looked up by equality, so the key type must support `=`: `integer`,
`real`, `boolean`, `string`, an enum, a [distinct type](distinct-types.md), or an
`Option`, `Result`, record, or enum whose payloads and fields all compare (see
[equality](../basics/operators.md)). Arrays, dictionaries, callables, tasks,
channels, and aggregates containing them are rejected as key types. A generic
key type parameter needs the `Comparable` constraint, for example
`function Index<K: Comparable>(Key: K): dict of K to integer`.

Dictionary writes require a mutable binding:

```pascal
var
  Counts: dict of string to integer := ['A': 1];

begin
  Counts['A'] := 2;
  Counts['B'] := 3;
end.
```

Dictionaries provide import-free dot operations such as `Length`, `ContainsKey`, `Get`, `Keys`, `Values`, and `Remove` — see [`Dictionary operations`](dictionary-operations.md).

## See also

- [For-in](../control-flow/for-in.md)
