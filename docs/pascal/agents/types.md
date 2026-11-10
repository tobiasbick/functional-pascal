# Types, units, and state

Read this when modeling data or sharing declarations across source files.

## Records and identity

A source record is a sealed CLR reference type with read-only properties.
Unmarked fields and methods are private to the record. Mark exposed members
and interface implementations `public`. Construct with
new Point(X := 1, Y := 2), or positional arguments in field order. Explicit
arguments evaluate in written order; omitted defaults run afterward in field
order, once per construction. Defaults use declaration scope and cannot read self.

Any private field makes the all-fields constructor private; expose a static
factory when needed. Instance methods use read-only self; static methods have no
self. Fields remain immutable even on a var record binding. A var can instead
be rebound to an updated record: Value with X := 3; end with. Updates evaluate
the receiver once and preserve unmentioned/private fields without rerunning
defaults. Assignment shares references, and updates are shallow.

Record equality compares fields with EqualityComparer<T>.Default. Arrays in
fields compare by identity, and mutable objects are not recursively frozen or
cloned. Imported structs retain CLR value semantics; imported classes retain
reference semantics. A source record is not a mutable CLR struct or an arbitrary
C# record. Generated CLR member names cannot be redeclared by source members.

## Aliases, domains, enums, and generics

`type Label = string;` is transparent and creates no CLR type. Use
`type UserId = distinct long;` for nominal identity. Construct with new UserId(7L),
unwrap with cast<long>(Id) or Id.Value. Distinct values have explicit base
conversions and same-domain comparisons, without automatic arithmetic/member
forwarding. Distinct string wrappers reject null.

`type Port = ushort range 1..65535;` adds checked inclusive bounds. Construction/
casts validate constants at compile time and runtime values with CLR exceptions.
A range widens to its base for arithmetic; explicitly reconstruct to validate a
result. Its default, including in a new array, is the lower bound. Separate
domain declarations retain separate identities even when their bounds match.

An enum is a real CLR enum with an integral base and named constants; unnamed
underlying values remain possible. Use a variant for cases with payloads; see
[patterns](patterns-errors.md). Records/variants may be recursive through references
and generic: Box<T>, Tree<T>, or Dictionary<string, int>. Type arguments uniformly
use angle brackets. Source records/variants are invariant. Constraints use where
clauses; [functions](functions.md) explains them and inference.

```pascal
program AgentTypes;

uses System as Sys, System.Console as Console;

type UserId = distinct long;
type Port = ushort range 1..65535;
type Box<T> = record
  public Value: T;
end record;
type Point = record
  public X: int;
  public Y: int := 2;
end record;

begin
  const Original: Point := new Point(X := 1);
  const Moved: Point := Original with X := 4; end with;
  const Id: UserId := new UserId(7L);
  const Wrapped: Box<UserId> := new Box<UserId>(Id);
  const Ports: array of Port := new Port[1];
  if Original.X <> 1 or Moved.X <> 4 or Moved.Y <> 2 or
    cast<long>(Wrapped.Value) <> 7L or Ports[0].Value <> 1 then
    raise new Sys.InvalidOperationException('Incorrect record or domain behavior');
  end if;
  Console.WriteLine('Types verified');
end;
end program;
```

## Units and global initialization

A unit has a qualified identity, imports/declarations, and end unit; it has no
entry block or separate initialization/finalization section. Top-level routines,
types, and initialized globals are private unless public. Public signatures may
not expose less-accessible types. Program free declarations cannot be public.

Imports always have aliases: uses Agent.Values as Values. An import exposes
qualified names, not unqualified members, and does not restore a dependency.
Aliases cannot be shadowed. Supply application units as sources/project
references and CLR libraries as resolved references. A library has units and
no program; an executable has exactly one program plus optional units.

The following two sources share the documented project label and compile together:

```pascal project=agent-units
unit Agent.Values;

public const Step: int := 2;
var Total: int := 0;

public function Advance(): int;
begin
  Total := Total + Step;
  return Total;
end function;

end unit;
```

```pascal project=agent-units
program AgentUnits;

uses Agent.Values as Values, System as Sys, System.Console as Console;

begin
  if Values.Advance() <> 2 or Values.Advance() <> 4 then
    raise new Sys.InvalidOperationException('Unit state was not shared');
  end if;
  Console.WriteLine('Units verified');
end;
end program;
```

Globals become static readonly/mutable fields with one CLR type initializer.
Initializers execute in written order on first active use, not application-wide
eager unit order. Forward/cyclic reads can observe CLR default storage. A global
const is a runtime readonly field, not a constant pattern/default/bound expression.
Shared state needs ordinary .NET synchronization.

Read [types](../language/types/README.md), [data domains](../language/types/data-domains.md),
and [program structure](../program-structure/README.md) for full semantics.
For exported metadata and cross-language identities, use
[CLR mapping](../../specs/clr-mapping.md).
