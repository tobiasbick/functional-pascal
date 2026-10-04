# For-in

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`for_in_stmt`).

## Array iteration

Iterates over each element of an array. The loop variable is immutable.

```pascal
uses Std.Console as Console;

const Names: array of (string) := ['Alice', 'Bob', 'Charlie'];
for Name: string in Names do
  Console.WriteLn(Name);
end for;
```

The element type must match the array's element type:

```pascal
uses Std.Console as Console;

const Scores: array of (integer) := [10, 20, 30];
for S: integer in Scores do
  Console.WriteLn(S);
end for;
```

## Dict key iteration

Iterates over the **keys** of a `dict of (K, V)` in insertion order. The loop
variable receives each key; values can be looked up via the key inside the body.
Import `uses Std.Dictionaries as Dictionaries;` when calling dictionary routines;
iteration and indexing themselves need no library call.

```pascal
uses Std.Console as Console;

uses Std.Dictionaries as Dictionaries;
uses Std.Conv as Conv;

  const Ages: dict of (string, integer) := ['Alice': 30, 'Bob': 25];
  for Name: string in Ages do
    Console.WriteLn((Name + ': ') + Conv.IntToStr(Ages[Name]));
  end for;
```

The loop variable type must match the dict's key type. Iterating an empty dict executes the body zero times. `break` and `continue` work as usual.

This is separate from the `in` membership operator in expressions. In `for K: string in Ages`, `in` introduces iteration. In `'Alice' in Ages`, `in` returns whether the dictionary contains that key.

```pascal
uses Std.Console as Console;

// Print only keys whose value exceeds 10
for K: string in Ages do
  begin
    if Ages[K] <= 10 then
      continue;
    end if;

    Console.WriteLn(K);
  end;
end for;
```

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [Dictionaries](../types/dictionaries.md)
- [Break and continue](break-continue.md)
