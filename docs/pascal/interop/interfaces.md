# Declaring and implementing interfaces

Interfaces describe nominal CLR contracts. Declare one with `type Name =
interface ... end interface;`, then list it after a record's `implements`.
Source and imported interfaces use the same dispatch and implementation rules.
A record needs an explicit implements declaration even when its members match.

## Declarations

```pascal
program InterfaceSources;

uses System as Sys, System.Console as Console;

type ISource<T> = interface
  Name: string;
  function Read(): T;
end interface;

type IDisposableSource<T> = interface extends ISource<T>, Sys.IDisposable
end interface;

type ConstantSource<T> = record implements IDisposableSource<T>
  Name: string;
  Value: T;

  function Read(): T;
  begin return self.Value; end function;

  procedure Dispose();
  begin end procedure;
end record;

begin
  using const Source: IDisposableSource<int> := new ConstantSource<int>('answer', 42);
  if Source.Name <> 'answer' or Source.Read() <> 42 then
    raise new Sys.InvalidOperationException('Incorrect interface dispatch');
  end if;
  Console.WriteLine(Source.Read());
end;
end program;
```

Top-level visibility follows ordinary type rules. Public declarations belong in
units. Members are public abstract instance signatures; explicit `public` is
optional. A property such as `Name: string;` has a getter and no storage or
initializer. Function/procedure signatures end after their semicolon and any
method constraints. They have no body or `end function`/`end procedure`.
Empty interfaces are allowed.

Generic type constraints follow `=` and precede `interface`. Method constraints
follow the signature, as on implemented routines:

```pascal fragment
public type IFactory<T> =
where T: class;
interface
  function Create(): T;
  function Echo<U>(Value: U): U;
  where U: class;
end interface;
```

Interfaces are invariant. Overloads, generic methods, optional/params parameters,
ref/out/in modes, delegate types, and nullability follow the existing routine
rules. A Task/Task<T> result expresses an asynchronous API; `async` belongs on
the implementing function. Contracts belong on implementations and still execute
when called through an interface, including from another .NET language.

Private/static members, constructors, bodies, contracts, events, setters, indexers,
variance, and default implementations cannot be declared in source interfaces.
Ordinary source attributes target the actual interface, method, or property.

## Inheritance and implementation

`extends` lists distinct non-nullable source or imported interfaces. Classes,
duplicate bases, inaccessible types, and inheritance cycles are errors. Generic
arguments substitute through the complete inherited graph. A diamond that reaches
the same declaring member has one slot. Distinct inherited slots may share a
compatible record implementation. Incompatible results, parameter modes, generic
constraints, and method/property/accessor collisions are errors. Access to
distinct equally applicable inherited members is ambiguous; cast to a declaring
interface to select a slot. Declaration order does not select one.

The record header lists interfaces after `implements`, separated by commas.
Public instance methods satisfy compatible interface members. Public read-only
record properties may satisfy matching getter-only interface properties; private
members and static methods do not satisfy instance contracts. Signatures match
CLR names, types, generic arity, parameter modes, compatible generic constraints,
and nullable safety. Implementations may accept more nullable inputs or return
less nullable results. Parameter labels need not match. Calls through an interface
use that interface's labels, defaults, and params declaration.

Every required abstract member must be implemented, including inherited members.
Existing imported default implementations may be used. Explicit-interface
implementations, setter properties, indexers, events, static abstract members,
and byref-like implementations cannot be authored by a record. Such an imported
interface can still be consumed where its signatures are supported. Unsupported
inherited requirements produce diagnostics even through a source interface.

Source declarations emit ordinary nested CLR interfaces with abstract methods,
getter metadata, base interfaces, generic constraints, nullable annotations, and
XML documentation identities. C# and other FPAS assemblies consume these through
normal references. No interface runtime wrapper is required.

## See also

- [Complete comparer and JSON example](../libraries/json.md#example)
- [Records](../language/types/records.md)
- [Generic data types](../language/types/generics.md)
- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
