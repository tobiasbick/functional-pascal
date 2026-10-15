# For-in

The code fragments use `uses Std.Console as Console;`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`for_in_stmt`).

The loop body is a nonempty statement list with its own local scope, closed
by `end for;`. Each statement ends with `;`; use `null;` for no action.

## Evaluation and source protection

The iterable expression is evaluated exactly once whenever execution enters the
loop, including for an empty collection. Each new entry into a nested or repeated
loop evaluates its source again. `break` and `continue` retain their ordinary
innermost-loop behavior.

Arrays visit element values in order. Dictionaries visit keys in insertion
order. Both the loop binding and the source's writable storage root are read-only
throughout the body. FP3046 rejects assignment to the source or its contents,
reassignment, InPlace receivers and explicit var arguments, including indirect
writes through helpers, closures, callbacks and imported routines. A callable
whose write effects cannot be proven safe is rejected while a writable source
is protected. The diagnostic suggests moving the write after the loop or
iterating an explicit independent copy.

Parentheses preserve the source root. For a record field or array element,
the containing root is protected too: iterating `State.Items` protects `State`,
including ancestor replacement and writes to other parts of that root. Nested
loops protect every active source root. Lexically shadowed bindings and independent
value copies have different roots and remain writable. Each protection ends with
its loop body; creating a closure without invoking it is not a source write.

A var parameter can alias a global, a captured variable or another reference.
If a write through either root could reach the protected source and their
disjointness cannot be proven, FP3046 rejects it. Independent local value copies
remain writable. This rule applies in both directions: iterating a var parameter
while writing a possible foreign alias, and iterating foreign storage while
writing a possible var-parameter alias.

Computed iterable values such as `Values.Keys()` or `Items.Filter(Predicate)`
are independent results, so their input roots are not protected in the body.
The complete iterable expression, including index selection and its calls, runs
before that loop's protection begins.

Iteration follows ordinary value-copy semantics. Channels still refer to the same
resource, and copied closures still share their captured environment. Iteration
does not deep-clone these resources or provide synchronization between tasks.

## Array iteration

Iterates over each element of an array. The loop variable is immutable.

```pascal id=example-1 fragment context=body-console
const Names: array of string := ['Alice', 'Bob', 'Charlie'];
for Name: string in Names do
  Console.WriteLn(Name);
end for;
```

The element type must match the array's element type:

```pascal id=example-2 fragment context=body-console
const Scores: array of integer := [10, 20, 30];
for S: integer in Scores do
  Console.WriteLn(S);
end for;
```

The source can be changed after the loop:

```pascal id=example-3 fragment context=body-console
var Items: array of integer := [1, 2, 3];
for Item: integer in Items do
  Console.WriteLn(Item);
end for;
Items.PushInPlace(9);
// Prints 1, 2, 3. Items is now [1, 2, 3, 9].
```

To change one array while visiting its original values, make the source copy
explicit. The copy retains ordinary value independence:

```pascal id=example-4 fragment context=body
var Items: array of integer := [1, 2, 3];
const Source := Items;
for Item: integer in Source do
  Items.PushInPlace(Item);
end for;
// Source remains [1, 2, 3]. Items is now [1, 2, 3, 1, 2, 3].
```

## Dict key iteration

Iterates over the **keys** of a `dict of K to V` in insertion order. The loop variable receives each key; values can be looked up via the key inside the body. No import is required.

```pascal id=example-5 fragment context=header
uses Std.Conv as Conv, Std.Console as Console;
begin
const Ages: dict of string to integer := ['Alice': 30, 'Bob': 25];

for Name: string in Ages do
  Console.WriteLn(Name + ': ' + Conv.IntToStr(Ages[Name]));
end for;

end.
```

The loop variable type must match the dict's key type. Iterating an empty dict executes the body zero times. `break` and `continue` work as usual.

Adding, removing or reinserting keys, replacing values, or assigning another
dictionary to the source is a compiler error during the body. Ordinary lookups
remain readable:

```pascal id=example-6 fragment context=body-console-conv
var Scores: dict of string to integer := ['Ada': 1, 'Grace': 2];
for Name: string in Scores do
  Console.WriteLn(Name + ': ' + Conv.IntToStr(Scores[Name]));
end for;
Scores['Grace'] := 20;
// Prints Ada: 1 and Grace: 2. Scores can be changed after the loop.
```

An explicit dictionary copy or `Scores.Keys()` produces an independent source
when changes to `Scores` are required. Lookups into that separate, changing
dictionary retain ordinary missing-key failures; use `Scores.Get(Name)` when
absence is expected. The protected source itself cannot lose a key in the body.

This is separate from the `in` membership operator in expressions. In `for K: string in Ages`, `in` introduces iteration. In `'Alice' in Ages`, `in` returns whether the dictionary contains that key.

```pascal id=example-7 fragment context=ages-loop
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
