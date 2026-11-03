# Implemented CLR objects, nullable types, and resources

This milestone extends the [compiler subset](first-program.md) using the existing
language syntax and direct CIL emission. The [file-reading example](../../examples/read-file/README.md)
uses System.IO.StreamReader through a normal `.fpasproj`. It consumes real .NET
types and methods from reference metadata, with no Fpas.Core dependency.

## Types and members

The compiler now represents CLR type identity and nullability separately from
primitive arithmetic kinds. Source types include all [CLR primitive aliases](scalar-types.md),
object, supported imported non-generic classes/interfaces/structs, and T?.
Reference T? uses nullable annotations; value T? emits actual System.Nullable<T>.
Byref-like types and System.Void as a value are rejected.

Namespace and type aliases resolve against supplied references and source units.
`uses System.IO as IO;` permits `IO.StreamReader` type names and `IO.Path.Combine`
static calls. A namespace/type identity ambiguity is diagnosed. Imports do not
discover packages or source files. Inheritance/interface relationships and
reference forwarders are resolved from the selected metadata.

Implemented member access includes constructors, chained instance calls, public
static/instance fields and non-indexed properties, and field/property assignment
where writable. Overloads use identity, numeric widening, reference
assignability, and boxing; competing candidates remain errors. Arguments and
receivers evaluate in source order. Virtual/interface calls dispatch through
callvirt. Struct receivers use addresses for writable storage and defensive copies
for read-only positions; source const still does not freeze reference objects.

Reference upcasts preserve identity, explicit reference casts use CLR checks,
and boxing/unboxing use actual CLR types. Existing checked arithmetic remains.
Nullable value construction, HasValue/Value, casts, widening, and coalescing use
Nullable<T> metadata/methods. Nullable arithmetic requires explicit unwrapping;
general lifted operator support is not claimed.

## Nullability and exported APIs

Nullability is read from parameter/return/property/field annotations and nullable
contexts. Oblivious reference results remain conservatively nullable.
Unconditional MaybeNull/NotNull return and AllowNull/DisallowNull parameter
contracts are read. Conditional postcondition attributes are not used to prove
flow facts in this implementation.

Null initializes only nullable values/references. A nullable reference cannot
be dereferenced or assigned/passed/returned as non-null without validation.
Stable local/parameter null tests refine if branches and short-circuit and/or
operands. Assignments invalidate/update facts; branch joins retain common facts.
Loops account for assigned locals at their back edge, and exception handlers
use conservative incoming facts. Repeated property reads do not establish
non-nullness. `??` evaluates the fallback only when the left value is absent.

Public source signatures emit proper CLR class/struct/Nullable<T> signatures and
NullableAttribute metadata on nullable reference parameters/results, with the
existing non-null container context. C# tests consume nullable strings/integers
and a StringBuilder API with warnings treated as errors.

## Exceptions and disposal

try/catch/finally, ordered typed/catch-all handlers, when filters, raise, and
raise-without-expression are implemented through CLR exception regions. Catch
types must derive from Exception; earlier unfiltered catches cannot hide later
handlers. A rethrow requires an active catch body and preserves the original
stack. Control transfers cannot leave finally; loops inside finally may use
their own break/continue. A rethrow from finally requires a catch nested inside
that finally, rather than an outer catch alone.

`using const Name: T := Expression;` protects the rest of its enclosing lexical
scope with try/finally and selects IDisposable. The initializer runs once before
its cleanup region. Reverse-order cleanup runs for normal exits, return,
break/continue, and exceptions. Null resources skip disposal; value-type resources
use constrained interface dispatch without a boxed replacement. Failed later
acquisition and throwing cleanup still release earlier acquisitions. CLR filters
retain first-pass ordering before stack unwinding, and filter exceptions count
as false. Cleanup exceptions follow ordinary CLR replacement behavior.

FP3013 reports unsafe nullable conversions/dereferences, FP3015 inaccessible
property accessors, FP3016 non-disposable resources, and FP3017 invalid exception
handlers/control transfers. Existing type/member/metadata diagnostic codes remain.

## Verification and remaining work

The xUnit suite executes BCL object/property/struct calls, nullable flow and
Nullable<T> operations, C# virtual/interface APIs, and exported nullable metadata.
Resource fixtures record acquisition, filter, body, and cleanup events to check
order on every exit, failed acquisition, cleanup failure, and struct cleanup.
Rethrow tests check an original non-inlined CLR frame. SDK tests build/run the
StreamReader example and retain the existing mixed-project and Mandelbrot cases.

[Arrays/indexers, args, char, closed CLR generic types, and enumeration](arrays-and-collections.md)
are implemented, as are [callable values, closures, and explicit generic CLR calls](functions-and-closures.md).
[Source generic routines](generic-routines.md), [records](records.md), and
[unions](unions-and-matching.md), including [generic data types](generic-types.md), are also implemented.
[Numbers and scalar domains](scalar-types.md) are implemented. Nullable lifted operators and conditional nullable contracts
remain later work. [XML documentation](xml-documentation.md) is implemented.
[Async/await using and portable PDBs](overloads-interfaces-debugging-and-async.md) are implemented. This is
partial CLR interoperability, not complete access to every .NET signature.
