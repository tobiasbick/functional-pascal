# Calling and implementing .NET APIs

Read this when using collections, libraries, interfaces, events, or attributes.

## Discover and consume actual APIs

Import a CLR namespace for generic types, or alias a nongeneric CLR type directly.
For source imports see [types and units](types.md#units-and-global-initialization).
Project/package references supply metadata. Use exact case-sensitive CLR names
and generic arities. `new Type(Arguments)` constructs an
accessible class/struct/record or constrained type. Fields, properties, indexers,
and ordinary instance/static methods retain .NET meaning. Read-only accessors/
storage cannot be assigned; a const reference can still point to a mutable collection.

Use `List of T`, `Dictionary of (K, V)`, Stream, Regex, JsonSerializer, HttpClient, and
other .NET APIs directly. Extension methods use static form: import
System.Linq.Enumerable and call Enumerable.Where/Select with typed delegates.
Member access does not discover extension methods. Inspect the referenced API
signature rather than inventing a wrapper or guessing parameter labels.

Imported structs are values. Mutation operates on writable receiver storage;
readonly struct receivers use defensive copies. References use normal virtual/
interface dispatch and CLR boxing/casts. Foreign reference annotations are
contracts; unannotated results are conservatively nullable. `typeof(T)` produces
the actual Type without constructing an instance or running its type initializer.

CLR operators resolve from both operand types and call the selected CLR method.
For example, DateTimeOffset subtraction yields TimeSpan. Arrays and references
without an applicable equality operator compare by identity. Value-type operators have
[lifted nullable forms](../language/types/nullable-types.md#lifted-value-operators).

## Protocols, interfaces, and attributes

Declare nominal CLR interfaces with `type Source of T = protocol ... end protocol;`.
Use `Name: T;` for getter-only properties and bodyless function/procedure
signatures for methods. Optional `extends` lists protocols or imported interfaces.
Type constraints precede `protocol`. Method constraints follow signatures.
Members are public abstract instance members. Generic protocols are invariant.
Read [protocols](../interop/interfaces.md) for inheritance and ambiguity rules.

Records can implement protocols and imported CLR interfaces with compatible public
instance methods and getter-only properties. List them after `implements`. Required
inherited abstract members must be satisfied; applicable default implementations
remain usable. Labels can differ but CLR types, modes, generic constraints, and
nullable safety must match. Setter/indexer/event/static-abstract implementations
cannot be authored by a record. Protocol declarations cannot contain bodies,
contracts, private/static members, setters, indexers, or events. Task-returning
signatures support async implementations; async is only on the implementation.

Place attributes on emitted types, routines, record fields or protocol properties.
Transparent aliases and globals cannot carry attributes. Use the complete
attribute type name, including `Attribute`. Positional arguments select its constructor; subsequent
named arguments set public writable properties/fields. Values must be legal CLR
metadata constants, closed typeof values, or supported array literals. Record
field annotations target emitted properties. Assembly/parameter/return/backing-
field target syntax is outside the baseline.

```pascal
program AgentInterop;

uses System as Sys;
uses System.Collections.Generic as C;
uses System.Console as Console;
uses System.Linq.Enumerable as Enumerable;
uses System.Text.Json as Json;
uses System.Text.Json.Serialization as Serialization;

type Descending = record implements C.IComparer of int
  public function Compare(Left: int; Right: int): int;
  begin
    return Right.CompareTo(Left);
  end function;
end record;

type Item = record
  [Serialization.JsonPropertyNameAttribute('item_name')]
  public Name: string;
end record;

begin
  const Values: C.List of int := new C.List of int();
  Values.Add(1);
  Values.Add(3);
  Values.Add(2);
  Values.Sort(new Descending());
  const Count: int := Enumerable.Count of int(Values);
  const Text: string := Json.JsonSerializer.Serialize of Item(new Item(Name := 'entry'));
  if Values[0] <> 3 or Count <> 3 or not Text.Contains('item_name')
    or typeof(int).Name <> 'Int32'
  then
    raise new Sys.InvalidOperationException('Incorrect CLR interoperability');
  end if;

  Console.WriteLine('Interop verified');
end;

end program;
```

## Events and boundaries

Subscribe/unsubscribe invokes the actual add/remove accessor, evaluating the
receiver before the handler, once each. Store the delegate when removal must
refer to the same handler. Event selections are not readable delegate values.
Subscription does not schedule automatic unsubscription or wait for an already
dispatched handler. Use explicit cleanup where the lifetime requires it.

```pascal
program AgentEvents;

uses System as Sys;
uses System.Collections.ObjectModel as C;
uses System.Collections.Specialized as Events;
uses System.Console as Console;

begin
  var Changes: int := 0;
  const Items: C.ObservableCollection of int := new C.ObservableCollection of int();
  const Handler: Events.NotifyCollectionChangedEventHandler := procedure(Sender: object?; Event: Events.NotifyCollectionChangedEventArgs) begin
    Changes := Changes + 1;
  end procedure;
  subscribe Items.CollectionChanged with Handler;
  Items.Add(1);
  unsubscribe Items.CollectionChanged with Handler;
  Items.Add(2);
  if Changes <> 1 then
    raise new Sys.InvalidOperationException('Incorrect event lifetime');
  end if;

  Console.WriteLine('Events verified');
end;

end program;
```

Imported methods and constructors accept vectors or array literals for by-value
`Span of T` and `ReadOnlySpan of T` parameters. Writable spans require the exact
element type. See [span parameters](../interop/supported-signatures.md#span-parameters)
for read-only conversions and overload selection. Imported `unmanaged` generics accept unmanaged value types. Other ref
structs, pointers, ref returns, init-only setters, and unsupported
modifiers/calling conventions receive diagnostics.
Use an ordinary supported API overload; runtime reflection is not a compiler
fallback. Nested CLR type paths put each segment's own arguments on that segment,
as in `(N.Outer of string).Inner of int`. `(N.Outer of string).Leaf` inherits its
owner's arguments. Imported static abstract/virtual interface
members dispatch through constrained type parameters, including generic operators,
`T.Zero`, and `T.CreateChecked of int`. Read [generic constraints](../language/functions/generic-routines.md).

Before changing public identities/signatures/representations, read the binding
[interop rules](../interop/README.md) and [CLR mapping](../../specs/clr-mapping.md).
Read [attributes](../interop/attributes.md) and
[events](../interop/events.md) for their precise supported contracts.

For tag, image and audio project references and FPAS calls, see [media libraries](../libraries/media/README.md).
