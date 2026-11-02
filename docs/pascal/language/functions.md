# Functions, delegates, and generics

## Declarations and calls

```pascal
public function Add(Left: int; Right: int): int;
begin
  return Left + Right;
end function;

procedure Show(Text: string);
begin
  Console.WriteLine(Text);
end procedure;
```

Parameters are separated by `;`, call arguments by `,`. Every routine has
parentheses, even without parameters. A function declares a result type and
returns a value on every normal path. A procedure returns no value and may use
`return;`. An expression-bodied declared routine is not part of the baseline.

Optional `requires` and `ensures` clauses appear after any `where` constraints
and before `begin`. They check routine entry and successful exit in all build
modes; see [contracts](contracts.md) for typed result bindings, expression limits,
cleanup ordering, and async behavior.

Contiguous standalone `//` comments immediately before a declaration document it
as Markdown. Structured Parameters, Type parameters, Returns, and Remarks sections
map to the optional .NET XML sidecar; see [comments and API documentation](comments.md).
Documentation does not alter a routine's signature or its executable contracts.

`Add(1, 2)` and `Add(Right := 2, Left := 1)` are valid. Positional arguments may
precede named arguments, but not follow them. Arguments evaluate once in written
order, even when names reorder their receiving parameters. Labels match the
selected declaration's parameter names exactly. Duplicate arguments are errors.

Parameters are immutable bindings by default, not deeply immutable objects.
Default parameter values use `Name: T := ConstantExpression`; required parameters
must precede optional parameters. Defaults must be CLR-compatible compile-time
constants and are inserted at the call site. `const` declarations with runtime
initializers do not qualify as default expressions.

Consume a function result or write `discard Expression;`. A bare value-producing
call statement is an error. A procedure/CLR void call is a statement. Discarding
a task does not wait, cancel, supervise, or detach it from a language owner:
there is no such owner. See [concurrency](concurrency.md).

## Overloads

Free routines and instance/static record methods may overload by CLR-compatible signatures.
Return type alone does not distinguish overloads. Parameter labels do not
distinguish signatures; static versus instance alone also does not distinguish
source overloads. `ref`, `in`, and `out` alone cannot distinguish overloads
whose underlying byref parameter types are identical.

Resolution uses argument count, names, explicit generic arguments, applicable
implicit conversions, and CLR/C# better-function-member rules. A method return
type alone never selects an overload. No applicable candidate or multiple best
candidates is a diagnostic listing the relevant signatures. Use explicit type
arguments or casts to resolve an ambiguity; do not choose by declaration order.

## Managed parameter references

```pascal
procedure Increment(ref Value: int);
begin
  Value := Value + 1;
end procedure;
```

- `ref Value: T` aliases writable caller storage; call with `ref Storage`.
- `out Value: T` aliases storage the callee must assign before normal return;
  call with `out Storage`. The callee cannot read it before assignment.
- `in Value: T` is a read-only managed reference; call with `in Storage`.
- An unmodified parameter passes by value.

Calls require addressable storage of the exact element type, with no numeric
conversion. Locals passed to `out` must still be declared with an initializer.
An ordinary CLR property is not addressable storage. Writable fields and array
elements may be passed as appropriate. Source `const` bindings are not writable
`ref`/`out` targets.

Byref parameters cannot be captured, returned by reference, stored in objects, or
used as async parameters. Async calls with ordinary byref arguments are allowed
only if the invoked method's CLR signature itself supports that synchronous call;
the caller must not keep a managed reference across suspension. No pointer,
`ref struct`, ref return, or byref-local syntax is provided.

`params Values: T[]` may be the final by-value parameter. A call may supply an
existing array or expanded element arguments. Expanded arguments allocate one
array. The same rules apply to imported CLR `ParamArray` parameters.

## First-class functions and closures

```pascal
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

A named routine or lambda converts to a compatible CLR delegate. Context resolves
an overloaded routine group; an unresolved group cannot be stored. With no other
delegate target, callable types map to `System.Func<...>` / `System.Action<...>`
within those delegate families' supported arities. Use an imported delegate type
for other signatures. Converting an already constructed delegate object to an
unrelated delegate type is not an implicit conversion.

Captured immutable bindings retain their values; captured local `var` bindings
share compiler-generated cells between closures. Capturing an object does not
copy it. There are no task-specific transfer restrictions or hidden snapshots.
Concurrent access to mutable captures requires normal .NET synchronization.
Closures may outlive their declaring call. Capturing a byref parameter is invalid.

## Generic routines and constraints

```pascal
function Identity<T>(Value: T): T;
begin
  return Value;
end function;

function Compare<T>(Left: T; Right: T): int;
where T: Sys.IComparable<T>;
begin
  return Left.CompareTo(Right);
end function;
```

Generic arguments use `<...>` in declarations, types, and calls:
`Identity<int>(42)`. Calls may infer routine type arguments from supplied argument
types. Expected result type alone does not infer routine type arguments. Union
constructor inference also uses expected type as defined in [types](types.md).
Fully qualified and nested generic CLR types are supported.

Each `where T: ...;` follows the routine header or the type's `=` and precedes
the body/type definition. Constraints may include a base class, interfaces,
`class`, `class?`, `struct`, or `new()`. They follow CLR/C# combination rules,
with `new()` last. A record constraint example is:

```pascal
type Box<T> =
where T: class;
record
  Value: T;
end record;
```

`new T()` requires an appropriate constructor constraint. There are no special
`Numeric`, `Printable`, `Comparable`, or `Error` constraints. Use actual .NET
interfaces and conversions. Applicable operators declared by constrained imported
interfaces use CLR static-interface dispatch; unsupported constraints must be
diagnosed, never erased into unchecked object operations.

## Visibility, recursion, and effects

Free declarations are private to their unit unless `public`. Record members are
public within their record's accessibility unless explicitly private. Private
members are accessible only from inside the record. A static method has no self
and does not capture an instance. Static factories may construct private record
state. Signatures are visible throughout their lexical declaration scope,
permitting mutual recursion. Binding values
initialize in source order; reads before definite assignment are errors.

Functions may perform I/O, mutate objects, and throw exceptions. No purity effect
system is implied by `function`. Contract conditions use a restricted read-only
expression subset; they do not declare the whole function pure. Use contracts
for executable expectations, ordinary CLR exceptions for failures, and explicit
domain outcomes where appropriate.
Tail-call optimization may be performed, but stack safety is not guaranteed by
the language; use loops for unbounded iteration.
