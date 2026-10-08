# Records

Records group related data together.

Records may refer to themselves through an aggregate field such as
`Children: array of Element`. These recursive record types are valid in ordinary variables,
function return types, first-class function signatures, and generic routine inference.

Types and record member signatures are available regardless of declaration
order within a unit or program. Mutually recursive stored fields must admit
finite construction; see [type declaration order](declaration-order.md).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_type`, `typed_record_construction`).

## Declaring a record

A record declaration ends with `end record;`. Its methods use `end function;` or
`end procedure;` according to their declaration kind.

```pascal
type Point = record
  X: real;
  Y: real;
end record;
```

## Creating a record

Call the record type with named fields:

```pascal
const P: Point := Point(X := 0.0, Y := 5.0);
```

Typed construction is the only way to create a record value; there is no
`record ... end` literal (FP2017). Construction requires named fields;
`Point(0.0, 5.0)` is an error. Unknown,
duplicate, missing required, and incorrectly typed fields are errors. Names
are case-insensitive. Fields may be supplied in any order. Methods
and events are not stored fields and cannot be initialized here.
Field values are copied; `var` arguments are not accepted.

Supplied values are evaluated once, in written order. Missing defaults run
afterwards, in field declaration order. Explicit values replace their defaults.
An empty record, or one whose fields all have defaults, permits `TypeName()`.
The result can be nested in another construction or followed by field access
and method calls, such as `Point(X := 1.0, Y := 2.0).X`.

Type aliases construct the original nominal type with the same defaults and
visibility rules:

```pascal
type Position = Point;
const Q: Position := Position(Y := 2.0, X := 1.0);
```

Imported types support both unit qualification and import aliases, for example
`Model.Point(X := 1, Y := 2)` after `uses MyApp.Model as Model`. Call targets
use ordinary lexical lookup: a nearer variable or routine named `Point` keeps
its meaning; an invalid call does not fall back to an outer record type.
Types and routines cannot share a case-insensitive name in the same scope.
Concrete record construction also works inside generic routines.

## Type identity and compatibility

Each named record declaration defines a distinct type. Two records are not compatible merely
because they contain fields with the same names and types. Assignments, arguments, and return
values must use the same record declaration or an alias of that declaration.

```pascal
type Point = record
  X: integer;
  Y: integer;
end record;

type Size = record
  X: integer;
  Y: integer;
end record;

type PointAlias = Point;

const P: Point := Point(X := 1, Y := 2);
const A: PointAlias := P; // Valid: PointAlias names the Point declaration.
const S: Size := P; // Error: Point and Size are distinct declarations.
```

Two values of the same record type compare with `=` and `<>` field by field when every field
compares; see [Operators](../basics/operators.md). A record with an array, dictionary, or callable
field has no whole-value equality.

## Accessing fields

```pascal
const PosX: real := P.X;
```

## Field visibility

Fields in records declared by a unit are private by default. Write `public`
directly before each field that importing units may access. FPAS has no
visibility sections and no explicit `private` keyword.

```pascal
unit MyApp.Counters;

public type Counter = record
  Value: integer;
  public Step: integer;
end record;
end unit;
```

Code in `MyApp.Counters` may read and write `Value`. Importing units may use
`Step`, but cannot name `Value`. An explicit `public` modifier has the same
meaning for that field.

A named record with at least one private field can be constructed only inside
its declaring unit, even if all private fields have default
values. Importers obtain such values from public functions or static functions.
They may copy received values and use record updates for public fields; private
fields are preserved and cannot be named in an update.

Record member visibility is valid only for records declared in unit files.
Functions, procedures, and events use the same private-default rule.

## Immutability

Record instances follow the same immutability rules as variables. A `var` record allows field reassignment:

```pascal
var P: Point := Point(X := 1.0, Y := 2.0);

begin
  P.X := 10.0; // Valid — P is mutable
end.
```

## Default field values

Defaults are checked at the record declaration's source position. They may
use preceding values and routines, and types declared anywhere in the unit or
program. Later values and free routines remain unavailable.

A field declaration may include a default value using `:=`. Construction
substitutes omitted defaults automatically. Defaults retain their
declaration environment when a caller shadows names. Fields without a default
must always be supplied. Defaults exported through unit interfaces must be
scalar constant expressions; transparent exported aliases preserve them.

```pascal
type Config = record
  Host: string := 'localhost';
  Port: integer := 8080;
  Debug: boolean := false;
end record;
```

Omitting defaulted fields:

```pascal
const C: Config := Config(); // Host='localhost', Port=8080, Debug=false
const D: Config := Config(Port := 9000); // Host='localhost', Debug=false
```

Explicitly providing a value overrides the default:

```pascal
const E: Config := Config(Host := 'example.com', Port := 443, Debug := true);
```

Fields without a default remain required:

```pascal
type Vertex = record
  Id: integer; // Required
  X: integer := 0; // Optional
  Y: integer := 0; // Optional
end record;

const V: Vertex := Vertex(
  Id := 7
); // X=0, Y=0 from defaults
```

## See also

- [Record methods](record-methods.md)
- [Visibility](../../program-structure/visibility.md)
- [Record events](record-events.md)
- [Record update](record-update.md)
- [Read-only parameters](../functions/parameters.md)
