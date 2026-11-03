# Repeat and typed counting loops

The compiler implements the [planned loop syntax](../pascal/language/control-flow.md)
with direct CIL branches and the existing async state-machine lowering. The
[loops example](../../examples/loops/README.md) executes an async subrange counter,
a repeat/until loop, and an unsigned counter ending at ulong.MaxValue.

## Counting loops

For/to/downto accepts sbyte, byte, short, ushort, int, uint, long, ulong, nint,
nuint, char, transparent aliases of these types, and declared integral subranges.
Nullable counters, enums, distinct scalars, floating/decimal values, and unknown
generic numeric parameters are not counting-loop types.

Lower and upper expressions evaluate once in written order and convert to the
declared counter type. Subrange endpoints require that domain type or an explicit
checked construction; raw base values do not implicitly become domain values.
Bounds are inclusive and a reversed interval executes no iterations. The fresh
counter is readonly in its body, including for ref/out argument checks.

The bound loop carries typed comparison, endpoint, and step expressions shared
by synchronous emission and async lowering. Comparisons preserve signedness.
Small integral steps promote/narrow normally; subrange steps unwrap their base
and perform checked construction. Equality with the endpoint is checked before
the step, so neither overflow nor a value outside a subrange is produced after
the final iteration. Continue completes cleanup and reaches this guarded step.

Closures capture each iteration's counter value. Mutable body-local captures
receive a fresh shared cell on each iteration. Native integer widths follow the
execution architecture, including runtime IntPtr/UIntPtr extremes.

## Repeat/until

```pascal
var Count: int := 0;
repeat
  Count := Count + 1;
  if Count = 1 then continue; end if;
until Count = 3;
```

The body executes before the Boolean condition. False repeats and true exits.
Every iteration has a lexical body scope; its declarations are unavailable in
the until expression and after the loop. The condition reads enclosing bindings
and runs after the body's using/finally cleanup, including awaited disposal.
Continue reaches the condition after cleanup. Break skips the condition;
return/raise retain ordinary cleanup and exception behavior. Failed cleanup
prevents the condition from running. Empty bodies are legal.

Flow analysis accounts for the first guaranteed iteration, returns, breaks,
continues, and a constant-false condition with no exit. Out-parameter analysis
merges all routes reaching the condition, includes finally assignments, and
tracks writes in the condition itself. It rejects reading or returning an out
parameter left unassigned on a possible exit.

Nullable facts are conservative at loop tests. Body assignments alone do not
establish a non-null invariant in until, since continue can bypass them. Use an
explicit check/coalescing in the condition or a non-nullable binding. Ref/out
writes and mutations of shared captures invalidate earlier facts.

## Verification and remaining scope

Tests execute every supported primitive width, signed/unsigned/native extremes,
subranges, empty/reversed intervals, once-only bound effects, per-iteration
captures, repeat control paths, out parameters, PDB condition locations, and
actual async suspension. Mixed C#/FPAS tests verify synchronous/async disposal,
finally ordering, break/continue/return behavior, condition failures, and cleanup
failures. The SDK builds and runs the checked-in example.

Authored interfaces/events, expression trees, arbitrary attributes, fully general
nested generic type paths, lifted numeric nullable arithmetic, async enumeration,
and LSP/query tooling remain outside this milestone.
