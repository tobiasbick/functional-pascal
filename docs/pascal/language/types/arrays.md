# Arrays and collections

`array of T` is a one-dimensional zero-based CLR vector. `array[,] of T` is
a rectangular two-dimensional array. Use one comma per additional dimension,
up to rank 32. `array of array of T` is jagged. Array type declarations nest
up to the parser's 256-level type limit.

`array of string?` allows null elements. `(array of string)?` allows a null
array. Combine these as `(array of string?)?`. A vector of nullable inner
arrays uses `array of (array of string)?`. Escape the reserved word with
`@array` to use it as a name.

Allocate with `new int[Count]` or `new double[Rows, Columns]`. Index with
`Values[I]` or `Grid[X, Y]`.
All dimensions are evaluated once and must be valid `int` lengths. Rectangular
rank is at most 32. Length is the total element count, returned as int.

New arrays use CLR default element initialization. For reference elements, an
allocation with potentially nonzero length requires a nullable element annotation;
use `new string?[Count]`. A non-null element vector can instead be constructed
with a fully initialized literal. The same rule applies to default-null elements
of rectangular arrays. Imported APIs retain their declared annotations and may
violate them like other external contracts. Jagged arrays can be constructed
from vector literals or through an element type alias; no special jagged `new`
suffix syntax is included in this baseline.
Literal zero dimensions permit non-null reference elements, as in `new string[0]`;
the compiler recognizes literal zeros rather than general constant expressions.

`[1, 2, 3]` is an array literal. Empty `[]` requires an expected vector type.
Array elements are mutable. Assigning an array shares it, including through
`const` bindings. Array bounds errors are CLR exceptions. Array literals are not
automatically lists or dictionaries.
Expected vector types can come from declarations, parameters, returns, casts,
and conditional branches. Otherwise nonempty literals need a common supported
element type; contextual inference follows [type inference](../functions/type-inference.md).

Use actual .NET collections:

```pascal fragment context=body
const Names: Collections.List<string> := new Collections.List<string>();
Names.Add('Alpha');
const Counts: Collections.Dictionary<string, int> := new Collections.Dictionary<string, int>();
Counts['Alpha'] := 10;
```

Members retain CLR meanings: `Length`, `Count`, `Add`, `TryGetValue`, `GetLength`,
and so on. There are no compiler-supplied `PushInPlace`, `dict`, or collection
snapshot semantics. Ordering and mutation during enumeration follow each API.

## Example

```pascal
program Collections;

uses System.Console as Console, System.Collections.Generic as Generic;

begin
  const Values: array of int := [1, 2, 3];
  const Shared: array of int := Values;
  Shared[0] := 9;
  Console.WriteLine(Values[0]); // 9: both bindings reference the same array

  const Names: Generic.List<string> := new Generic.List<string>();
  Names.Add('Alpha');
  Names.Add('Beta');
  for Name: string in Names do
    Console.WriteLine(Name);
  end for;

  const Counts: Generic.Dictionary<string, int> := new Generic.Dictionary<string, int>();
  Counts['Alpha'] := 10;
  var Count: int := 0;
  if Counts.TryGetValue('Alpha', out Count) then
    Console.WriteLine(Count);
  end if;

  var Parsed: int := 0;
  if int.TryParse('42', out Parsed) then
    Console.WriteLine(Parsed);
  end if;
end;

end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
