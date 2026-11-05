# First-class functions and closures

Pass behavior using ordinary CLR delegate values and typed lambdas.

```pascal fragment context=body
const Double: function(int): int :=
  function(Value: int): int => Value * 2;

const Print: procedure(string) :=
  procedure(Text: string) begin
    Console.WriteLine(Text);
  end procedure;
```

Callable types are `function(T1, T2): R` and `procedure(T1, T2)`. Parameter names
do not occur in callable types; calls through these types are positional.
`function(): R` and `procedure()` represent zero-argument callables.
The baseline supports by-value parameters in callable type syntax. Use an
explicit imported delegate type such as `Sys.Func<int, int>?` for a nullable
delegate or a delegate array. Suffixes on a function's result type belong to
that result, not to the callable itself.

Anonymous functions have typed parameters, a result annotation, and either `=>`
with one expression or a full `begin ... end function` body. Anonymous procedures
use `begin ... end procedure`. Their parameter names remain local to the body.
Named nested routines use the ordinary declaration syntax inside a routine body.
Their signatures are visible throughout the declaring lexical block, allowing
forward and mutually recursive calls. A declaration has no runtime statement
effect. Captured bindings must be initialized before a call or conversion to a
delegate; see [nested routines](nested-routines.md#captures-and-initialization).

A named routine or lambda converts to a compatible CLR delegate. Context resolves
an overloaded routine group; an unresolved group cannot be stored. With no other
delegate target, callable types map to `System.Func<...>` / `System.Action<...>`
within those delegate families' supported arities. Use an imported delegate type
for other signatures. Converting an already constructed delegate object to an
unrelated delegate type is not an implicit conversion.

The shorthand supports at most 16 by-value parameters. Delegate signatures use
identity/reference conversions and supported CLR variance with nullable safety;
numeric signature adaptation is not inserted. A virtual method group evaluates
its receiver once and preserves virtual dispatch. A struct method group captures
a boxed snapshot; a lambda capturing a mutable struct variable instead uses its
shared storage.

Captured immutable bindings retain their values; captured local `var` bindings
share compiler-generated cells between closures. Capturing an object does not
copy it. There are no task-specific transfer restrictions or hidden snapshots.
Concurrent access to mutable captures requires normal .NET synchronization.
Closures may outlive their declaring call. Capturing a byref parameter is invalid.
Capturing a resource does not extend its usable lifetime after its using scope
disposes it. A lambda owns its own return/loop/catch/finally context; break,
continue, and rethrow cannot target constructs in the enclosing routine.

## Example

```pascal
program Functions;

uses System.Console as Console;

function Identity<T>(Value: T): T;
begin
  return Value;
end function;

function Apply(Value: int; Transform: function(int): int): int;
begin
  return Transform(Value);
end function;

function MakeCounter(): function(): int;
begin
  var Count: int := 0;
  return function(): int begin
    Count := Count + 1;
    return Count;
  end function;
end function;

begin
  const Double: function(int): int := function(Value: int): int => Value * 2;
  const Counter: function(): int := MakeCounter();
  Console.WriteLine(Identity<string>('Functions first'));
  Console.WriteLine(Apply(21, Double)); // 42
  Console.WriteLine(Counter());        // 1
  Console.WriteLine(Counter());        // 2
end;
end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
