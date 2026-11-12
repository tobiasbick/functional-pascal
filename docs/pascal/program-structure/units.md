# Units and global values

```pascal project=geometry id=geometry
unit Numerics.Geometry;

public type Point = record
  public X: double;
  public Y: double;
end record;

public function Translate(Value: Point; DX: double; DY: double): Point;
begin
  return Value with
    X := Value.X + DX;
    Y := Value.Y + DY;
  end with;
end function;
end unit;
```

A unit has no executable entry body. Top-level declarations are types, functions,
procedures, and initialized `const`/`var` bindings. There are no separate interface
sections or unit initialization/finalization blocks. Its required terminator is
`end unit;`.

Declarations are private unless marked `public`. Public signatures cannot expose
less-accessible types. Free functions are emitted as static methods on a unit
container. Nested language types remain ordinary CLR types. The emitted identities
are specified in [CLR mapping](../../specs/clr-mapping.md).

Signatures are available across the compilation without declaration-order lookup
restrictions. Value initializers execute once in source order in the unit's CLR
type initializer on first active use. Do not assume an application-wide eager
initialization order. Cyclic initializer dependencies retain CLR initialization
behavior; there is no source initialization scheduler.

### Global values

```pascal
unit Numerics.Statistics;

public const MaximumSamples: int := 1_000;

var TotalSamples: long := 0L;

public procedure RecordSample();
begin
  TotalSamples := TotalSamples + 1L;
end procedure;

public function SampleCount(): long;
begin
  return TotalSamples;
end function;
end unit;
```

Each top-level binding requires an explicit type and initializer. Const emits a
static readonly field; var emits a mutable static field. Both default to unit-private
visibility and may be public. An imported public var is writable through its unit
alias and may be passed by ref/out under the ordinary addressability rules.

Initializers for both kinds run in written order in the same CLR type initializer.
There is no automatic locking, atomic increment, or task isolation. Shared global
state needs ordinary .NET synchronization. Contract conditions cannot read global
vars; use initialized parameters/captures or compile-time constants instead.
Source global const values are runtime readonly fields rather than CLR metadata
constants, even when initialized with literals. They do not qualify as constant
patterns, optional defaults, enum/subrange bounds, or contract operands.
Forward/cyclic initialization reads follow CLR default-storage semantics.

Private globals are accessible only in their unit, including its records and
lambdas. Local/parameter/record members take precedence over outer globals.
Lambdas and async routines read the same static storage, rather than captured
snapshots. Names cannot collide with another declaration/import or entry args.
Public field types must be publicly accessible through nested aliases, arrays,
and generic arguments. Field reads do not acquire stable nullable facts; copy or
coalesce a nullable field into a local before dereferencing it.

## Initialization and failure

A container with globals has one private static .cctor with SpecialName and
RTSpecialName flags, and no BeforeFieldInit flag. Initializers run once in their
written order within that source, including declarations interleaved with types
or routines. A static method call or field access triggers ordinary CLR type
initialization. Merely importing a unit, inspecting its metadata, or obtaining
its type does not initialize it.

There is no eager application-wide ordering or generated dependency scheduler.
Forward and cyclic reads use CLR default storage while initialization is still
in progress; reference storage can therefore be null before its initializer has
run. A failed initializer follows CLR TypeInitializationException behavior,
retains its original inner exception, and is not retried. Concurrent first
accesses use the CLR's type-initialization synchronization. Later shared field
mutation needs ordinary .NET synchronization.

Program globals initialize before the entry body and do not have access to args.
Initializers are synchronous and cannot await. They can construct ordinary
Task/delegate values without a language task owner or automatic resource cleanup.
No source unit initialization/finalization blocks are introduced.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
