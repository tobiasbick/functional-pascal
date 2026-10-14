# For-in

The code fragments use `uses Std.Console as Console;`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`for_in_stmt`).

The loop body is a nonempty statement list with its own local scope, closed
by `end for;`. Each statement ends with `;`; use `null;` for no action.

## Evaluation and snapshots

The iterable expression is evaluated exactly once whenever execution enters the
loop, including for an empty collection. Each new entry into a nested or repeated
loop captures a new snapshot. `break` and `continue` retain their ordinary
innermost-loop behavior.

Arrays capture their original element values in order. Dictionaries capture
their original keys in insertion order. Mutation or reassignment of the source
binding does not change the visited sequence. The loop binding is read-only.

Snapshots follow ordinary value-copy semantics. Channels still refer to the same
resource, and copied closures still share their captured environment. Iteration
does not deep-clone these resources or provide synchronization between tasks.

## Array iteration

Iterates over each element of an array. The loop variable is immutable.

```pascal
const Names: array of string := ['Alice', 'Bob', 'Charlie'];
for Name: string in Names do
  Console.WriteLn(Name);
end for;
```

The element type must match the array's element type:

```pascal
const Scores: array of integer := [10, 20, 30];
for S: integer in Scores do
  Console.WriteLn(S);
end for;
```

Source mutation remains visible after the loop, while appended elements are not
visited by that execution:

```pascal
var Items: array of integer := [1, 2, 3];
for Item: integer in Items do
  Console.WriteLn(Item);
  Items.PushInPlace(9);
end for;
// Prints 1, 2, 3. Items is now [1, 2, 3, 9, 9, 9].
```

Removing elements, replacing element values or assigning a different array to
`Items` likewise leaves the original visited values unchanged.

## Dict key iteration

Iterates over the **keys** of a `dict of K to V` in insertion order. The loop variable receives each key; values can be looked up via the key inside the body. No import is required.

```pascal
uses Std.Conv as Conv;

const Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];

for Name: string in Ages do
  Console.WriteLn(Name + ': ' + Conv.IntToStr(Ages[Name]));
end for;
```

The loop variable type must match the dict's key type. Iterating an empty dict executes the body zero times. `break` and `continue` work as usual.

Adding, removing or reinserting keys, or replacing the source dictionary, does
not change the captured key sequence. Separate lookups in the body read the
current dictionary:

```pascal
var Scores: dict of string to integer := ['Ada': 1, 'Grace': 2];
for Name: string in Scores do
  Scores['Grace'] := 20;
  Scores['Linus'] := 3;
  Console.WriteLn(Name + ': ' + Conv.IntToStr(Scores[Name]));
end for;
// Prints Ada: 1 and Grace: 20. Linus is not visited.
```

Removing a captured key does not remove it from the iteration. A later
`Scores[Name]` for that key raises the normal missing-key runtime error; use
`Scores.Get(Name)` when absence is expected. The loop captures keys, not the
results of future body lookups.

This is separate from the `in` membership operator in expressions. In `for K: string in Ages`, `in` introduces iteration. In `'Alice' in Ages`, `in` returns whether the dictionary contains that key.

```pascal
// Print only keys whose value exceeds 10
for K: string in Ages do
  if Ages[K] <= 10 then
    continue;
  end if;

  Console.WriteLn(K);
end for;
```

## See also

- [Arrays intro](../basics/arrays-intro.md)
- [Dictionaries](../types/dictionaries.md)
- [Break and continue](break-continue.md)
