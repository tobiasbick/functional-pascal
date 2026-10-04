# Dictionaries

`dict of (K, V)` stores key-value pairs. Keys keep insertion order when iterated
with `for-in`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `dict of`, dict literals, indexing).

```pascal
const Ages: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
const Empty: dict of (string, integer) := [:];
const AliceAge: integer := Ages['Alice'];

```

Dictionary writes require a mutable binding:

```pascal
 var Counts: dict of (string, integer) := ['A': 1];

begin
  Counts['A'] := 2;
  Counts['B'] := 3;
end;
```

Use `Std.Dictionaries` for helpers such as `Length`, `ContainsKey`, `Get`, `Keys`, `Values`, and `Remove` — see [`Std.Dictionaries`](../../std/collections/dict.md).

Two dictionaries compare with `=` and `<>` when their keys and values support
structural equality. Dictionaries compare by key/value mapping,
independently of insertion order. Indexing, membership and `Std.Dictionaries` key
operations use the same
structural key comparison. Resource, task or callable components prevent equality.

## Key types and construction

Keys must support structural equality. Scalars, equatable records/enums,
Option/Result and equatable collections are supported. Resource, task and callable
components are rejected, including inside a record, collection or transparent
alias. Generic key parameters require `Equatable` or a stronger constraint.
Dictionary values can contain callables or resources; only their keys have this
construction restriction.

Every literal key and value is evaluated once in written order, key before value.
When several keys compare equal, their first position is retained and the last
value replaces earlier values:

```pascal
const Values: dict of (string, integer) := ['A': 1, 'B': 2, 'A': 42];
// Values contains A = 42 followed by B = 2.
```

Structural keys use the same equality as lookup and updates, including dictionary
keys whose internal insertion order differs. Real equality is IEEE equality:
signed zeros compare equal, while NaN keys do not compare equal to themselves or
other NaN keys.

## See also

- [For-in](../control-flow/for-in.md)
