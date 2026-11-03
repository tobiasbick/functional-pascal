# Loops

Repeat calculations or enumerate actual CLR collections with typed bindings.

```pascal fragment
for Index: int := 0 to Values.Length - 1 do
  Console.WriteLine(Values[Index]);
end for;

for Item: string in Items do
  Console.WriteLine(Item);
end for;

while Ready do
  Work();
end while;

repeat
  Work();
until Done;
```

Repeat executes its body at least once, then evaluates a bool condition: false
repeats and true exits. Each iteration has a body-local scope; those declarations
are unavailable in the until condition and after the loop. The condition runs
after body cleanup. Continue reaches this condition after cleanup; break exits
without evaluating it. Return/raise and failed cleanup also skip the condition.

Counting loops have primitive integral (including char and native integers) or integral-subrange variables and inclusive
bounds. For a subrange, endpoints must already have that type or use explicit
checked construction. Increment/decrement stays within its bounds and stops
before stepping beyond the final value. Distinct scalars are not loop counters.
Both bounds evaluate once in written order. `downto` counts downward. A reversed range runs
zero times. The loop terminates before incrementing past its final value, even
at the integral type's minimum/maximum. The loop variable is a new read-only
binding scoped to the loop body; closures capture that iteration's value.

`for Item: T in Source` requires the explicit element type. Omitting `: T` is an
error. The source evaluates once. Vectors and strings use their ordinary
CLR elements (`char` for strings). Other sources must provide a supported
`IEnumerable<T>` or non-generic `IEnumerable` view; non-generic elements are
`object?`. Each element must be implicitly convertible to the declared loop
variable type. Use an explicit projection or cast for other conversions. Dispose
an acquired disposable enumerator on every exit.
Rectangular arrays follow CLR enumeration order and retain their known element
type. There are no user-defined duck-typed enumeration patterns in the baseline.

Enumeration preserves collection behavior. Modifying a collection can invalidate
its enumerator; the compiler does not insert a snapshot or prohibit all aliasing.

`break;` and `continue;` affect the innermost loop. `return` and `raise` may also
leave loops. All these exits honor applicable `finally` and
resource cleanup. Empty bodies are legal; `null;` is not an empty statement.

Async enumeration is outside the initial scope. Consume channels through their
ordinary async APIs; see [concurrency](../concurrency/README.md).

## Example

```pascal
program LoopValues;

uses System as Sys, System.Console as Console;

begin
  const Values: int[] := [2, 3];
  var Total: int := 0;
  for Value: int in Values do Total := Total + Value; end for;
  for Index: int := 0 to 1 do Total := Total + Index; end for;
  repeat Total := Total - 1; until Total = 5;
  while Total < 7 do Total := Total + 1; end while;
  if Total <> 7 then raise new Sys.InvalidOperationException('Incorrect loop result'); end if;
  Console.WriteLine(Total);
end;
end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
