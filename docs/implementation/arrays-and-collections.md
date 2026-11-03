# Arrays, collections, and enumeration

This milestone extends the [CLR object subset](clr-objects-and-resources.md).
It emits actual CLR arrays, closed generic types, and ordinary interface calls;
there is no collection wrapper library or generated C#.

## Source surface

```pascal
program Collections;
uses System.Collections.Generic as C, System.Console as Console;
begin
  const Names: string[] := ['Ada', 'Mara'];
  const Counts: int[,] := new int[2, 3];
  Counts[1, 2] := 7;
  const Groups: int[][] := [[], [1, 2]];
  const Optional: string?[] := new string?[3];
  const Values: C.List<string> := new C.List<string>();
  for Name: string in Names do Values.Add(Name); end for;
  Values[0] := 'Updated';
  for Name: string in Values do Console.WriteLine(Name); end for;
  for Argument: string in args do Console.WriteLine(Argument); end for;
end;
end program;
```

All bindings, parameters, returns, and loop variables still require explicit
types. The program entry receives immutable args: string[] with the supplied
arguments only; helper routines do not receive it implicitly. Its CLR entry
method takes an ordinary string[] parameter. Array elements remain mutable even
through a const binding; assignment shares the array.

Vectors T[], rectangular T[,]/T[,,]/... arrays of rank at most 32, and jagged
arrays are supported. Allocation accepts one int length per dimension, evaluated
once in source order. CLR bounds/allocation exceptions remain CLR exceptions.
Length returns the total number of elements as int. Rectangular arrays use real
CLR constructor/Get/Set/Address signatures; writable struct elements retain their
storage when calling mutating methods or assigning fields.

Literals [1, 2, 3] construct fully initialized vectors. Empty literals need an
expected vector type from a declaration, source parameter, return, cast, or
conditional branch context. Otherwise nonempty literals find a common supported
element type. An allocation that can have nonzero size requires nullable reference
elements, e.g. new string?[Count]; new string[0] is accepted. The implementation
recognizes literal zero dimensions, not general constant expressions. Jagged
arrays can be built from nested literals; source type aliases are supported.
Special jagged new suffixes are outside the baseline syntax.

Strings expose their CLR UTF-16 indexer returning char. #65 denotes one UTF-16
code unit; character codes must be in 0..65535. char widens to int/double,
numeric casts honor checked/unchecked rules, and string+char/char+string
concatenate. Surrogate pairs occupy two indexes and two enumeration elements.

## CLR generic types and indexers

Namespace-qualified types such as C.List<string>, C.Dictionary<string, int>, and
nested arguments such as C.List<C.List<int>> are resolved by exact CLR arity.
Constructors, ordinary methods, fields, properties, inherited members, interface
members, and public indexers substitute the owner's actual type arguments.
Public nested CLR types returned by APIs, including Dictionary.Values and
List.GetEnumerator, retain their declaring-type identity in metadata.
Explicit nested type spelling with generic enclosing types remains pending.

The compiler checks imported class/class?, struct, new(), base/interface, and
nullable notnull constraints. Variance follows CLR in/out flags for reference
type arguments. Invariant type arguments retain their nullable annotations;
List<string> and List<string?> cannot be interchanged. Array conversions also
reject weakening nullable element annotations that would permit unsafe writes.

Indexing resolves the CLR DefaultMember property and appropriate public accessor.
Receiver, index arguments, and assignment value execute once in written order.
Getter and setter availability, argument conversions, and writable struct storage
are checked separately. No synthetic dictionary/list index operations are added.

Method/field MemberRefs keep the original generic signature (!0, etc.), with a
constructed owner TypeSpec. Public FPAS signatures expose actual arrays and CLR
generic instances. Recursive nullable annotations preserve distinctions such as
List<string?[]>? across C# and FPAS library boundaries. Oblivious external
reference annotations remain conservative; imported code can still violate its
own declared contracts.

## Enumeration and cleanup

The source of for Item: T in Source evaluates once. T is required and the loop
binding is read-only. Vectors and strings use cached-source index loops;
rectangular arrays retain the known element type and CLR row-major order.

Other collections use a unique compatible IEnumerable<T> view and IEnumerator<T>.
Each element must implicitly convert to T. A source with generic enumerable
views but no unique compatible one is rejected. Non-generic IEnumerable uses
object? elements and a runtime IDisposable check. Duck-typed and async enumeration
are not implemented.

Enumerator acquisition happens before its protected region. An acquired disposable
enumerator is released on completion, break, return, body/MoveNext/Current errors,
and outer exception unwinding. continue advances the same enumerator. A disposal
failure follows CLR exception replacement rules while outer cleanup still runs.
No snapshot is inserted: normal collection mutation invalidation remains visible.

## Verification and limits

xUnit exercises CLI arguments (including spaces and Unicode), arrays and aliasing,
rectangles/jagged arrays, UTF-16 chars, ordinary methods on closed generic types, inherited
members and fields, indexer evaluation order, nullable nested metadata through
C# and a second FPAS library, imported constraints, and enumeration failures and
cleanup. The SDK builds and runs the checked-in
[directory inventory](../../examples/directory-inventory/README.md) against an
isolated file tree and checks contents/counts and usage/I/O exit codes.

[Callable values, closures, and explicit generic CLR method calls](functions-and-closures.md)
are implemented, as are [records](records.md), [unions](unions-and-matching.md),
and [generic source routines/constraints](generic-routines.md).
[Generic records and unions](generic-types.md) now add reusable typed models.
[Type argument inference](type-argument-inference.md) now supports source/CLR calls
and contextual union constructors. [Contracts and CLR parameters](contracts-and-parameters.md)
cover synchronous checks and broader routine argument modes.
[Primitive aliases and scalar domains](scalar-types.md) and [XML documentation](xml-documentation.md) are implemented.
[Async and portable PDBs](overloads-interfaces-debugging-and-async.md) are implemented.
