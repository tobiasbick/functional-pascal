# Synchronous contracts and CLR parameters

The compiler implements synchronous `requires` and `ensures` on free routines,
record instance/static methods, generic routines, and block-bodied lambdas.
The language rules are in [contracts](../pascal/language/functions/contracts.md).
[Named nested routines](nested-routines.md) support these checks and parameter modes.
Authored interfaces are a [deferred extension](../future/language-gaps.md#deferred-declare-fpas-interfaces).
[Async routines](overloads-interfaces-debugging-and-async.md) now check requires
inside the state machine and ensures on the inner result after successful cleanup.

```pascal
procedure Add(ref Value: int; out Previous: int; in Delta: int);
requires Delta > 0;
ensures Value = Previous + Delta;
begin
  Previous := Value;
  Value := Value + Delta;
end procedure;
```

Requires clauses run in order before the body. Every successful return evaluates
and retains its result once, completes the body's `using`/`finally` cleanup, and
passes through one postcondition epilogue. Procedures use that epilogue for early
returns and fallthrough. Body, return-expression, and cleanup exceptions bypass
postconditions. False conditions throw `System.InvalidOperationException`;
exceptions raised by condition evaluation propagate unchanged. The checks live
outside the body's handlers and execute in Debug and Release, including calls
from C# and delegates. No support package or call-site rewriting is needed.

A typed `ensures (Returned: T): Condition;` binding is read-only, belongs to that
clause, and must match the declared result type. Body locals are unavailable to
contracts. Contract-only references establish closure captures. Conditions accept
built-in expressions, constants, record-field and selected union-payload reads,
arrays, string indexing/Length, and Nullable<T>.HasValue/Value. Arbitrary calls,
constructors, CLR getters, and generated aggregate equality are rejected.
FP3027 diagnoses invalid contract expressions and result bindings.

## Parameters and calls

Source routines and imported CLR signatures support `ref`, `out`, and `in`.
The call spells the same mode and supplies addressable storage of the exact
element type. `ref`/`out` require writable storage; `in` also accepts readonly
locals and fields. Properties are not addressable. Managed reference parameters
cannot be captured. A mutating struct method on an `in` receiver uses a defensive
copy. `out` must be assigned before reading and on every successful exit;
assignments in `finally` and forwarding through another `out` call count.

Optional source parameters use CLR-compatible constant defaults. Defaults are
inserted at the call site and emitted as ordinary Optional/HasDefault metadata.
A final `params Values: T[]` accepts an explicit array or expanded elements,
including no elements. Imported ParamArray metadata follows the same rules.
Named arguments match parameter names exactly; positional arguments precede
named arguments. Receivers and arguments evaluate once in written order even
when labels reorder the receiving parameter slots.

```pascal
function Sum(Seed: int := 1 + 2; params Values: int[]): int;
begin
  var Total: int := Seed;
  for Value: int in Values do Total := Total + Value; end for;
  return Total;
end function;
```

Generic inference accepts ordinary and expanded arguments, and exact managed
reference types. Imported methods and constructors use metadata defaults and
parameter names. Explicit imported delegate types support managed-reference
routines and lambdas; callable shorthand remains by-value. Anonymous functions
do not declare optional or params parameters. Source overload declarations are
supported. Byref locals/returns, pointers, and byref-like CLR types are outside the baseline.
FP3028 diagnoses invalid modes, defaults, storage, captures, and out flow.

## Verification

Compiler tests cover invalid contracts, typed result scope, invalid parameter
modes/defaults, out flow, readonly storage, named calls, and captures. Execution
tests cover early returns, retained results, cleanup order/failure, condition and
body exceptions, generic contracts, contract-only captures, nullable conditions,
array references, generic inference, defaults, and expanded arguments.

Mixed C#/FPAS projects verify callee checks, delegate calls, reflected parameter
metadata, optional/params calls, and readonly struct receivers in Debug and
Release. The full solution uses the same build/test workflow on Windows and Linux:

```sh
dotnet restore Fpas.slnx
dotnet build Fpas.slnx --no-restore --configuration Debug
dotnet test Fpas.slnx --no-build --no-restore --configuration Debug
```

The compiler reads stdin as strict UTF-8, as it does source files. Test subprocess
pipes use UTF-8 explicitly. Output assertions retain explicit LF characters
inside strings while using the host newline for Console.WriteLine terminators.
Test subprocesses disable MSBuild node reuse and the MSBuild server so build
workers do not retain redirected pipes after the command exits. Timeout reports
distinguish a running command from an exited command with open streams.
