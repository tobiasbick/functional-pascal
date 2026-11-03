# Functions, closures, and generic CLR methods

This milestone extends [arrays and collections](arrays-and-collections.md) using
the existing [function syntax](../pascal/language/functions.md). Values are actual
CLR delegates. Calls and closures compile directly to CIL, with no reflection
invocation, generated C#, or Fpas.Core runtime dependency.

## Callable values

```pascal
program Pipeline;
uses System.Console as Console, System.Linq.Enumerable as Seq,
  System.Collections.Generic as C;

function Apply(Transform: function(int): int; Value: int): int;
begin return Transform(Value); end function;

function Double(Value: int): int;
begin return Value * 2; end function;

begin
  const Transform: function(int): int := Double;
  Console.WriteLine(Apply(Transform, 3));
  const Threshold: int := 2;
  const Selected: C.IEnumerable<int> := Seq.Where<int>([1, 2, 3, 4],
    function(Value: int): bool => Value > Threshold);
  for Value: int in Selected do Console.WriteLine(Value); end for;
end;
end program;
```

Callable shorthand function(T1, T2): R maps to System.Func<T1, T2, R>;
procedure(T1, T2) maps to System.Action<T1, T2>. Zero parameters are supported;
the shorthand accepts at most 16 parameters. Values can be bound, passed,
returned, stored in arrays/collections, and invoked directly. Parameter/result
types remain mandatory. Nullable delegates and delegate arrays use explicit
imported types, such as Sys.Func<int, int>? or Sys.Action<string>[]. Suffixes on
a callable's result type belong to that result, as in the language reference.

Named source routines (including public unit routines) and supported CLR method
groups convert to compatible delegates. A CLR method group can be overloaded;
a delegate target chooses its best compatible member. Virtual instance method
groups preserve virtual dispatch and evaluate their receiver once. Struct method
groups capture a boxed copy, while a lambda capturing a mutable struct variable
operates on that variable's shared storage. Null instance receivers are rejected
unless the usual stable-binding flow check proves them non-null.

Typed lambdas support both function(Value: int): int => Value + 1 and
function(Value: int): int begin return Value + 1; end function. Anonymous
procedures use procedure(Text: string) begin ... end procedure. Return coverage,
explicit types, unused-result rules, and local visibility follow normal routines.
Lambda parameters use semicolon separators; callable type parameters use commas.

Delegate compatibility uses identity/reference conversions (including supported
CLR variance), preserving nullable parameter/result safety. Numeric signature
adaptation is not inserted. A lambda or routine group can target a custom CLR
delegate, e.g. Comparison<int> or Predicate<int>. An already constructed Func
value does not implicitly become an unrelated Predicate delegate object.

## Capture and control flow

Each lambda emits a private nested closure class in its source container, with a
constructor, captured fields, and an instance invocation method. Immutable
captures store their current values. Mutable captured locals use shared
System.Runtime.CompilerServices.StrongBox<T> cells, so sibling/nested closures and
the declaring routine observe the same variable. Captured local struct mutation
uses the cell's actual value address. Separate routine calls own separate cells.
Cells declared inside a loop are allocated per iteration; read-only loop bindings
are copied when the delegate is created, preserving each iteration's value.

Closures can escape their declaring call. Capturing a reference shares the object;
it does not extend a using resource's usable lifetime after disposal. Delegates
are constructed as ordinary objects; no identity cache or implicit synchronization
is provided. Mutable captures follow normal .NET synchronization requirements.

A lambda has its own return/loop/catch/finally context. It may be declared inside
an outer finally and return from its own body. break/continue cannot target an
outer routine's loop; rethrow requires a catch inside the lambda. Exceptions from
delegate invocation propagate directly, honoring the lambda's own CLR cleanup.

Nullable flow treats shared captures conservatively. Their initial value is not
assumed to hold when a deferred lambda eventually runs. Local checks can prove
non-nullness, but calls, property/indexer access, cleanup, and branch/loop joins
invalidate facts that a callback could change. Use an immutable local snapshot
when a stable value is needed across arbitrary callbacks.

## Generic CLR methods

Static and instance CLR generic calls support explicit type arguments:
Seq.Where<int>(Values, Predicate), Seq.Select<int, string>(Values, Transform),
and Seq.ToArray<string>(Source). Generic methods on closed generic owners retain
both independent sets of arguments. Supported generic method groups can also
convert to delegates with explicit type arguments.
[Call inference](type-argument-inference.md) now permits Where(Values, Predicate),
Select(Values, Transform), and ToArray(Source) when supplied types determine a
unique instantiation. Generic method groups retain explicit arguments.

Imported constraints are checked for class/class?, struct, new(), notnull, and
base/interface relationships. Unsupported constraints/signatures are rejected;
no object erasure hides a failed check. Nullable generic results retain actual
reference annotations and Nullable<T> value representations.

Emission keeps the generic MemberRef signature, including !!0 method parameters,
and emits MethodSpec tokens for concrete method arguments. CLR callbacks use
normal delegate Invoke calls. LINQ extension methods are invoked in static form;
there is no extension-method discovery or query syntax. Deferred LINQ execution,
re-enumeration, exceptions, and iterator disposal retain ordinary .NET behavior.

## Verification and remaining work

Tests execute higher functions, returned/nested closures, sibling mutable captures,
per-iteration values/cells, nullable captures, source/CLR method groups, virtual
and struct receivers, custom delegates, generic static/instance methods, callback
overloads, and deferred LINQ side effects. Mixed C#/FPAS builds exercise exported
Func signatures, captured state surviving a return, C# callbacks, and CLR generic
constraints. Negative tests cover missing annotations, signature mismatches,
nullable callbacks, stale nullable facts after callbacks/branches, ambiguous
overloads, and invalid control transfers.

The [directory inventory](../../examples/directory-inventory/README.md) now filters
through Where<string>, materializes paths, and transforms rows with Select. Its
block lambda captures the root directory and a mutable total; file reads occur
once while enumerating the rows. SDK tests check the actual file tree and totals.

[Named nested routines](nested-routines.md) are implemented. Expression trees and events remain pending.
[XML documentation](xml-documentation.md) is implemented.
[Async and portable PDBs](overloads-interfaces-debugging-and-async.md) are implemented. Callable
shorthand has the standard Func/Action arity limit. This milestone does not
implement the complete language baseline or all CLR signatures.
[Nongeneric records](records.md) now support methods, closures capturing self,
and named/default constructor arguments. [Unions and matching](unions-and-matching.md)
adds typed payload bindings that may be captured by returned lambdas.
[Source generic routines](generic-routines.md) now emit CLR type parameters and
constraints, with type-preserving generic closure classes.
[Generic data types](generic-types.md) add enclosing type parameters to record
methods and their closures, including methods with their own generic parameters.
