# For-in

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`for_in_stmt`).

## Array iteration

Iterates over each element of an array. The loop variable is immutable.

```pascal
var
  Names: array of string := ['Alice', 'Bob', 'Charlie'];

for Name: string in Names do
  WriteLn(Name);
```

The element type must match the array's element type:

```pascal
var
  Scores: array of integer := [10, 20, 30];

for S: integer in Scores do
  WriteLn(S);
```

## Dict key iteration

Iterates over the **keys** of a `dict of K to V` in insertion order. The loop variable receives each key; values can be looked up via the key inside the body. Requires `uses Std.Dict`.

```pascal
uses Std.Dict, Std.Conv;

var Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];

for Name: string in Ages do
  WriteLn(Name + ': ' + IntToStr(Ages[Name]));
```

The loop variable type must match the dict's key type. Iterating an empty dict executes the body zero times. `break` and `continue` work as usual.

This is separate from the `in` membership operator in expressions. In `for K: string in Ages`, `in` introduces iteration. In `'Alice' in Ages`, `in` returns whether the dictionary contains that key.

```pascal
{ Print only keys whose value exceeds 10 }
for K: string in Ages do
begin
  if Ages[K] <= 10 then continue;
  WriteLn(K);
end;
```

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [Dictionaries](../types/dictionaries.md)
- [Break and continue](break-continue.md)
