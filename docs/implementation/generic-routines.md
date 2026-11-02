# Generic source routines

This milestone extends [callable values](functions-and-closures.md) with generic
FPAS functions/procedures and record methods. Declarations emit ordinary CLR
generic methods, with no object-based type erasure or generated C#. The
[generic pipeline](../../examples/generic-pipeline/README.md) is an executable
example of Identity, Map, and Compose.

```pascal
unit GenericFunctions;
uses System as Sys;

public function Identity<T>(Value: T): T;
begin return Value; end function;

public function Compare<T>(Left: T; Right: T): int;
where T: Sys.IComparable<T>;
begin return Left.CompareTo(Right); end function;

public function Hold<T>(Value: T): function(): T;
begin return function(): T => Value; end function;

end unit;
```

## Declarations and calls

Put type parameters after the routine name. All value bindings, parameters, and
results still require explicit annotations. Type parameters are visible in the
signature, body, constraints, and nested lambdas. Free functions/procedures and
instance/static record methods support this syntax. Generic declarations are
bound before bodies, allowing recursion and cross-unit calls.

Calls can use explicit type arguments, such as
Identity<int>(42), OtherUnit.Identity<string>('text'), or
`const Copy: function(int): int := Identity<int>;`.
Without explicit arguments, [call inference](type-argument-inference.md) uses the
supplied value/callable types and then checks constraints. A generic body may
forward its own parameters to another source or CLR generic method. Routine
result types do not infer arguments; method groups keep explicit arguments even
when a delegate target is known. [Source overloads](overloads-interfaces-debugging-and-async.md)
use the same generic argument rules.

## Constraints

Where clauses follow the declaration header and precede begin. Each parameter
has at most one clause. Supported requirements are class, class?, struct, new(),
an accessible non-sealed CLR/source base class, imported instance interfaces,
and a non-cyclic type-parameter constraint such as `where T: U;`.
System.Enum constraints are supported, including `where T: struct, Sys.Enum;`.

Primary class/struct constraints must come first; a base/type-parameter
constraint precedes interfaces; new() must come last and cannot be combined
with struct. Duplicate, cyclic, or inconsistent constraints are errors. Nullable
reference annotations are retained on constraints. A struct-constrained
parameter cannot serve as another parameter's base constraint.

The same requirements are validated at source calls, when constructing CLR
generic types containing these parameters, and when forwarding them to CLR
methods. The compiler uses constraints to resolve instance members and interface
dispatch. Static-abstract interface requirements, unmanaged/byref-like
constraints, and generic-math operators remain unsupported and are rejected.
There are no built-in Numeric/Printable constraints or runtime reflection fallback.

`new T()` requires struct or an explicit new() requirement and takes no arguments.
It calls the actual CLR Activator.CreateInstance<T>(), preserving the CLR's
value/reference constructor behavior. Constraints are also emitted for C# and
other consumers to enforce independently.

## Nullability and type-independent code

An unconstrained T can be a value type, a non-null reference, or a nullable
argument. Returning/passing the same T preserves that type. It does not let a
body return null as an unannotated T or assume a value is safe to dereference.
A stable null check establishes non-nullness before member use or conversion
to a non-null object. Boxing to object? is available without that check.

Class and non-null interface/base constraints prove non-nullness. A class?
constraint permits nullable reference arguments and requires validation before
dereferencing a value of T. T? requires a reference or struct constraint;
unconstrained T? syntax remains invalid. Reference T? emits nullable annotations,
while struct T? uses the actual Nullable<T> representation. Nullable annotations
remain aligned through nested generic return types and assembly boundaries.

Nonempty new T[Count] cannot initialize a non-null reference element for every
possible T, so it requires a struct element constraint or an explicitly nullable
reference element annotation. Empty arrays and initialized array literals remain
available. The pipeline example uses List<U>.ToArray() to construct fully
initialized results without weakening element annotations.

Ordinary arithmetic on an unknown T is not numeric code; use supported concrete
types or constrained instance methods. Runtime `is T`, explicit boxing/unboxing,
and generic null/coalescing checks retain CLR behavior. Generic resources and
enumerables use constrained interface calls and normal cleanup. Specific case
patterns on an otherwise unknown T can be expressed after boxing to object?;
this milestone does not infer a union/constant domain for T from its call sites.

## Closures and CLR output

Generic signatures use CLR method parameters (!!0, !!1, ...), GenericParam rows,
GenericParamConstraint rows, and MethodSpec instantiations. Public signatures
cannot expose private types through constraints. Functions from another FPAS
project are consumed through ordinary metadata, exactly as imported CLR methods.

A lambda inside a generic routine becomes a private generic closure type. Its
fields and invocation body use that type's own parameters (!0, !1, ...), with
the enclosing method's actual arguments supplied at construction. Nested
closures and mutable StrongBox<T> captures retain those types and constraints.
Generic method groups become actual delegates. A struct method group captures
a boxed snapshot; a lambda over a mutable generic variable operates on its
shared storage. Class instances keep normal reference behavior.

The emitter uses constrained calls when a receiver may be a value or reference
type. It boxes or unboxes where CLR conversions require it, and keeps typed
array element operations. Generic functions require no Fpas.Core runtime support;
records/unions used alongside them keep their existing metadata dependency.

## Verification and remaining work

Tests cover generic recursion, Map/Compose, method groups, interface/base/dependent
constraints, new T(), null validation, Nullable<T>, nested closures, mutable
captures, struct/reference dispatch, generic record methods, resource cleanup,
and generic exception matching. Metadata tests inspect actual method/type
parameters and MethodSpec/TypeSpec signatures. Mixed builds call exported methods
from C# and from a second FPAS project, compare nullable metadata with a C#
equivalent, and verify nullable warnings in a C# consumer. The pipeline example
is built and executed through its real project.

[Generic records and unions](generic-types.md) now support typed data and methods
with both enclosing type parameters and their own method parameters.
[Type argument inference](type-argument-inference.md) now supports source/CLR
calls and union construction. Authored interfaces,
additional primitives, and LSP/query tooling
remain pending. [Contracts and CLR parameters](contracts-and-parameters.md) cover
synchronous contracts and ref/out/in/params/optional arguments.
[XML documentation](xml-documentation.md) is implemented.
[Source overloads, record CLR interfaces, async, and portable PDBs](overloads-interfaces-debugging-and-async.md)
are implemented.
The [complete generic syntax](../pascal/language/functions.md#generic-routines-and-constraints)
continues to specify the wider language goal. FP3026 diagnoses unsupported or
invalid source generic declarations/explicit argument arity; constraint failures
continue to use FP3018.
