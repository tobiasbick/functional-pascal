# Routines, callables, generics, and contracts

Read this when declaring functions/procedures, changing caller storage, or passing
behavior as values.

## Parameters and calls

Every routine has parentheses. Separate declared parameters with semicolons,
supplied arguments with commas. Functions declare their result type and return
on every normal path; procedures return no value and may use return;.
Consume returned values or explicitly use discard. Discarding a task does not
wait, cancel, or supervise it.

Positional arguments may precede named arguments; none may follow them. Labels
match the selected declaration exactly. Receiver/arguments execute once in
written order. Required parameters precede optional parameters. Optional defaults
must be legal CLR constants, not runtime const bindings. A final params Values:
`array of T` accepts an array or expanded elements, allocating one array when expanded.

Value parameters are read-only bindings. Ref aliases writable storage; out must
be assigned before reading and on every successful exit; in is a read-only managed
reference. Both declarations and calls spell the mode: Increment(ref Count).
Targets must be addressable storage of exactly the parameter type. Ordinary
properties are not addressable, and const bindings cannot be ref/out targets.
Managed references cannot be captured or used as async parameters.

Routines and record methods may overload by CLR-compatible signatures. Return
type, labels, nullability, or ref/in/out distinctions alone cannot distinguish
otherwise identical signatures. Resolution uses applicability and better-member
rules; explicit casts/type arguments can resolve an ambiguity.

## Functions as values

Callable shorthand is function(int): int or procedure(string), with types but
no parameter labels. Calls through shorthand types are positional. Named routines
and typed lambdas convert to compatible CLR delegates. An anonymous function uses
either => Expression or a full begin/end function body; procedures have a block
body. Use an imported delegate type for byref parameters, nullable delegates, or
signatures outside Func/Action arity limits.

Named nested routines are declarations inside a lexical body. Their signatures
are visible throughout that block, permitting forward and mutually recursive
calls. Put a routine's direct helpers after its constraints and contracts, before
its outer `begin`. Helpers in inner blocks or the program entry block go inside
that scope.
Captured bindings must be initialized before calls/delegate conversion. Immutable
captures retain values; captured var locals share cells and can outlive the call.
Capturing a reference does not copy the object. Mutable captures need ordinary
.NET synchronization when used concurrently.

## Type parameters and contracts

Use Identity<T>, Box<T>, and explicit calls such as Identity<int>(7). Routine
calls can infer arguments from supplied argument types, not only an expected
return type. Generic method groups require explicit arguments. Variant constructor
inference also uses the expected variant type. Binding annotations remain mandatory.

Where clauses follow a routine's header, or a type declaration's equals sign.
Use real CLR requirements: class, class?, struct, base/interface constraints,
type-parameter dependencies, and new() last. New T() requires a suitable
constructor constraint. An unknown T needs a matching imported interface for
numeric operators. For example, where T: N.INumber<T>; permits T.Zero,
T.CreateChecked(Value), and arithmetic through constrained static CLR dispatch.

After where clauses, place requires clauses followed by ensures clauses, then
local routine declarations and begin. Conditions are read-only bool expressions over permitted inputs/captures
and built-in operations. Arbitrary calls, CLR getters, foreign operators, and
generated aggregate equality are disallowed. A typed ensures binding names the
result only in that clause. It must have exactly the result type.

Requires runs before the body; ensures runs after successful body cleanup, on all
normal returns. False conditions throw InvalidOperationException in every build.
Body/cleanup faults skip ensures. Conditions do not snapshot old values or undo
effects. Async Task<T> ensures binds T, and failures fault the returned task.

```pascal
program AgentFunctions;

uses System as Sys, System.Console as Console;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

procedure Increase(ref Value: int);
ensures Value > 0;
begin
  Value := Value + 1;
end procedure;

function Add(Left: int; Right: int := 1): int;
ensures (Answer: int): Answer = Left + Right;
begin
  return Left + Right;
end function;

function MakeCounter(Start: int): function(): int;
  function Next(): int;
  begin
    Count := Count + 1;
    return Count;
  end function;
begin
  var Count: int := Start;
  return Next;
end function;

begin
  var Value: int := 1;
  Increase(ref Value);
  const Counter: function(): int := MakeCounter(Value);
  const Double: function(int): int := function(Item: int): int => Item * 2;
  if Counter() <> 3 or Counter() <> 4 or Double(Identity(7)) <> 14
    or Add(Right := 3, Left := 2) <> 5 or Add(2) <> 3
  then
    raise new Sys.InvalidOperationException('Incorrect call or closure behavior');
  end if;

  Console.WriteLine('Functions verified');
end;

end program;
```

Read [functions](../language/functions/README.md) for overloads and delegates,
[contracts](../language/functions/contracts.md) for condition/execution restrictions, and
[nested routines](../language/functions/nested-routines.md) for capture initialization.
