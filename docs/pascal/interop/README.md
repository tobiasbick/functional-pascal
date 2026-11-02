# .NET interoperability

The compiler reads assembly metadata and reference assemblies to bind .NET
symbols. It does not maintain a handwritten catalog of available library methods
or generate one FPAS wrapper per .NET API. Supported calls emit normal CIL calls,
constructors, field/property access, and interface dispatch.

## Names and construction

```pascal
uses System.Net.Http as Http;

// Inside a routine:
const Client: Http.HttpClient := new Http.HttpClient();
Client.Timeout := Sys.TimeSpan.FromSeconds(10.0);
```

`new Type(Arguments)` constructs an accessible CLR class, struct, language record,
or constrained generic type. Constructor overload resolution is the same as for
methods. Creating a struct with `new T()` uses its accessible parameterless
constructor where one exists; otherwise it uses permitted CLR value initialization.
The compiler does not expose arbitrary reference-type `default(T)` creation.

Access members by their exact metadata names. `Value.Property` invokes the getter;
`Value.Property := Expression` invokes an accessible setter. Read-only properties
cannot be assigned. Static members use the type name. Indexers use `Value[Index]`
or `Value[X, Y]`; array indexing is distinct from indexer overload resolution.

Source aliases and named arguments do not rename metadata. Named arguments use
the selected API's actual parameter names. Most interop examples use positional
arguments to avoid depending on third-party parameter naming conventions.

## Managed type behavior

Imported classes have reference semantics; imported structs have value semantics.
The compiler must preserve value-type receiver mutation and storage rules rather
than accidentally mutate a boxed copy. `const` bindings do not prohibit mutation
of referenced objects. A mutating method on a struct in a read-only receiver
position follows CLR/C# defensive-copy behavior; assigning the receiver's fields
directly still requires writable storage.

Imported inheritance and interface assignability participate in normal conversions
and overload resolution. Supporting imported class hierarchies does not require
user-authored inheritance syntax. A virtual call dispatches virtually; an
interface call dispatches through that interface. Boxing and unboxing follow
the rules in [types](../language/types.md).

Nullable annotations are read as described in [nullable types](../language/types.md#nullable-types).
Foreign code may violate its annotations; the compiler does not add a second
presence wrapper to method results. `CancellationToken`, `DateTime`, `Guid`, and other
structs are actual CLR values, not FPAS handles.

## Overloads, generics, and delegates

Use `<...>` for explicit generic arguments, including static methods:

```pascal
const Text: string := Json.JsonSerializer.Serialize<Player>(PlayerValue);
```

The compiler must check metadata constraints, accessibility, optional arguments,
`params`, named arguments, byref argument modes, and conversions before emission.
Selection is static. It does not fall back to runtime reflection or dynamic
invocation when overload resolution fails.

Compatible lambdas and named routines convert to CLR delegates. For instance,
`Items.Sort(function(A: int; B: int): int => B.CompareTo(A));` selects a compatible
Comparison<int> overload. A matching callable value is not automatically an
expression tree. Consumers requiring Expression<TDelegate> need a separately
specified feature.

Extension methods are callable in **static form** in the baseline. Import their
declaring type, for example System.Linq.Enumerable, and call
`Enumerable.Select<int, int>(Values, Transform)`. Automatic extension-method
discovery through dot syntax is not included. CLR instance methods remain normal
dot calls.

## Implementing interfaces with records

```pascal
type Descending = record implements Collections.IComparer<int>
  function Compare(Left: int; Right: int): int;
  begin
    return Right.CompareTo(Left);
  end function;
end record;
```

The record header lists imported interface types after `implements`, separated
by commas. Public instance methods satisfy compatible interface members. Public
read-only record properties may satisfy matching getter-only interface properties;
private members and static methods do not satisfy instance contracts.
Signatures match CLR names, types, generic arity, and parameter modes; parameter
labels need not match. Calling through the interface uses that interface's labels.

Every required abstract member must be implemented, including inherited members.
Existing default implementations may be used. Conflicting same-name members with
incompatible signatures produce an error; explicit-interface implementations are
not in the baseline. Setter properties, indexers, events, static abstract members,
and byref-like signatures cannot be authored as interface implementations in this
baseline. Such an interface can still be consumed where its signatures are
otherwise supported. There is no user-authored interface declaration syntax.

## Events

```pascal
const Handler: Timers.ElapsedEventHandler :=
  procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin
    Console.WriteLine('Tick');
  end procedure;

subscribe Timer.Elapsed with Handler;
unsubscribe Timer.Elapsed with Handler;
```

`subscribe EventReference with Delegate;` invokes the CLR add accessor.
`unsubscribe` invokes the remove accessor. Evaluate the event receiver before
the delegate, once each. Event references are special member selections, not
ordinary readable delegate values. Accessibility is checked on each accessor.

Store the delegate when later removal must use the same instance/target. The
compiler does not turn repeated lambda evaluation into a stable identity or
automatically remove subscriptions at scope exit. There is no source event
declaration in the initial language.

## Attributes

```pascal
type Player = record
  [Serialization.JsonPropertyNameAttribute('player_name')]
  Name: string;
end record;
```

Attribute lists precede a type, routine, or record field declaration. Specify the
exact attribute type name, including its Attribute suffix. Constructor arguments
are positional. Following named arguments set public writable attribute properties
or fields, using `Name := ConstantExpression`; they do not name constructor
parameters. Values must be legal CLR attribute constants, typeof expressions,
or supported arrays of those values.

A field annotation on a language record applies to its emitted property.
Routine attributes apply to its emitted method. AttributeUsage and constructor
binding are checked. Assembly/return/parameter/backing-field attribute targets are
not part of the initial source syntax. Compiler-generated nullability and parameter
metadata are emitted without requiring user-written attribute syntax.

## Resources and async APIs

Use actual IDisposable/IAsyncDisposable contracts through `using const` and
`await using const` declarations. Cleanup occurs on exiting their enclosing scope.
Await the supported .NET task families directly. There is no thread-per-async-call
translation. Exceptions, cancellation, cleanup order, and context capture follow
the rules in [errors and resources](../language/errors-and-resources.md) and
[concurrency](../language/concurrency.md).

## Explicit limits

The initial compiler must reject unsupported metadata with a specific diagnostic.
In particular, it must not pretend that ref structs/Span<T>, pointers, function
pointers, ref returns, unmanaged calling conventions, expression trees, arbitrary
custom awaiters, init-only object initializers, or dynamic binding are ordinary
supported calls. Consuming ordinary managed APIs is the initial interoperability
target; complete C# language and ABI feature coverage is not assumed.
