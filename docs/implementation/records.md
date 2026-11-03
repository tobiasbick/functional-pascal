# Records

This milestone implements nongeneric records from the
[type reference](../pascal/language/types.md#records). They compile directly to
sealed CLR reference classes nested in the program/unit container. No generated
C# or VM is involved. The [inventory model](../../examples/inventory-model/README.md)
is a complete executable example.

```pascal
unit Models;

public type Position = record
  Name: string;
  Quantity: int := 1;

  function Add(Amount: int): Position;
  begin
    return self with Quantity := self.Quantity + Amount; end with;
  end function;
end record;

public function Make(): Position;
begin
  return new Position(Name := 'Potion');
end function;

end unit;
```

## Construction and members

Fields require explicit types and are immutable. Records in units may be public
or private (the default); program records are private. Fields and methods are
public by default, with explicit public/private modifiers available. Instance
methods receive an immutable `self`. Static methods have no self. Methods can
call unit routines and other record methods, and return closures that capture
self. Private members are accessible only lexically inside that record,
including its lambdas and static factories.

The single all-fields constructor uses declaration order. FPAS calls accept
positional arguments followed by named arguments. Supplied expressions run once
in written order, followed by omitted defaults once in field declaration order.
Defaults bind in declaration scope, without self or instance fields, and can call
unit/static routines. Missing required fields, duplicate/unknown names, and
positional arguments after named arguments are errors. These named/default
arguments apply to record construction; ordinary routine arguments remain
positional and required.

Any private field makes the constructor private, even if that field has a
default. A public static factory inside the record can construct the value.
Defaults do not create additional constructor overloads: a record with fields
does not satisfy a CLR new() constraint just because every field has a default.

## Updates and equality

`Value with Quantity := 2; end with` evaluates the receiver once, then update
expressions once in written order, and returns a shallow copy. The receiver must
be a non-null FPAS record; ordinary .NET classes and C# records do not qualify.
Unmentioned fields, including private state, retain their values. Defaults do
not rerun. Private fields can only be updated inside the owning record.

Generated equality implements IEquatable<T>, typed Equals(T?), object.Equals,
GetHashCode, and the CLR equality/inequality operators. FPAS `=` and `<>` use
these operators. Fields use EqualityComparer<T>.Default, including record value
equality, .NET Double.NaN equality, and array reference identity. Identical
references are equal; null equals null and differs from a non-null record.
GetHashCode uses System.HashCode with the same field equality rules. Neither
equality nor copying recursively clones mutable objects held by fields.

Generated names (including Equals, GetHashCode, operators, and field getters)
cannot collide with source members. Public signatures cannot expose private
record types.

## CLR and assembly boundaries

The example exports Models.Position, consumed in C# as
`new Models.Position("Potion", 1)`, with getter-only Name and Quantity properties
and an instance Add method. C# supplies every constructor argument; computed FPAS
defaults are not CLR optional constants. Nullable field/parameter/result
annotations are emitted normally. A second FPAS project can import the unit or
record type and use its public constructors, defaults, methods, and with updates.

Records use two small metadata attributes in Fpas.Core, documented in the
[CLR mapping](../specs/clr-mapping.md#records-and-unions). Imported marker versions,
field order, helper ownership/signatures, visibility, and readonly property shape
are checked before construction/update lowering. Unknown versions or malformed
contracts produce FP4001. Helpers preserve private payloads without exposing
them, including private fields whose types are private records.

The SDK bundles the matching Fpas.Core.dll, resolves it for compilation, and
copies it as a runtime dependency through mixed FPAS/C# project builds. Normal
C# record consumers need no source reference to attribute types; code that names
those attributes directly must explicitly reference Fpas.Core. The CLI adds its
bundled Core as a reference unless one is already supplied. Direct compiler API
callers supply Core alongside their other references when declaring records.
Direct CLI deployments must include Fpas.Core.dll alongside other non-framework
dependencies when record metadata is inspected. Outputs without records
or unions do not acquire an emitted Fpas.Core assembly reference.

## Verification and remaining work

Tests execute constructors/defaults, updates, private factories and closures,
recursive/forward types, equality/hash collections, nulls, array identity, and
evaluation order. Metadata checks cover CLR shape, determinism, visibility, and
invalid imported contracts. Mixed-project tests build one FPAS model library,
a second FPAS consumer, and a C# host, including reflected runtime metadata.
The inventory example is built and executed through its actual project.

[Unions and matching](unions-and-matching.md) now supports domain cases whose
payloads can contain records. [Generic methods](generic-routines.md) and
[generic record declarations](generic-types.md) are implemented, including defaults,
copy updates, and closures with both type and method parameters.
[Source overloads, record CLR interfaces, async, and portable PDBs](overloads-interfaces-debugging-and-async.md)
are implemented, as is [XML documentation](xml-documentation.md).
[Primitive numbers and scalar domains](scalar-types.md) are implemented.
Authored interfaces and globals remain pending. The broader
[record-members design example](../pascal/examples/record-members.md) includes
several of those planned features and is not yet an executable compiler example.
