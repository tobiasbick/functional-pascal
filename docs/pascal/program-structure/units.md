# Units and global values

```pascal project=geometry
unit Game.Geometry;

public type Point = record
  X: double;
  Y: double;
end record;

public function Translate(Value: Point; DX: double; DY: double): Point;
begin
  return Value with X := Value.X + DX; Y := Value.Y + DY; end with;
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
unit Game.Statistics;

public const MaximumScore: int := 1_000;
var TotalMoves: long := 0L;

public procedure RecordMove();
begin
  TotalMoves := TotalMoves + 1L;
end procedure;

public function Moves(): long;
begin
  return TotalMoves;
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

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
