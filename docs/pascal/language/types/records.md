# Records

Records group related data together.

Records may refer to themselves through an aggregate field such as
`Children: array of (Element)`. These recursive record types are valid in ordinary variables,
function return types, first-class function signatures, and generic routine inference.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_type`, `record_construction`).

## Declaring a record

```pascal
type Point = record
  X: real;
  Y: real;
end record;

```

## Creating a record

```pascal
var P: Point := Point(X := 0.0, Y := 5.0);

```

Named construction resolves a declared record type or a concrete alias of it.
It also supports [generic records](generics.md#generic-records-and-enums).
Each field may appear at most once. Field names are
case-insensitive, so `X` and `x` identify the same field and cannot both be
specified.

Supplied fields evaluate once in written order. Omitted defaults then evaluate
once in declaration order. An empty construction such as `Settings()` requires
every field to have a default. Unknown fields, missing required fields, duplicate
fields and positional record arguments are errors.

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

var P: Point := Point(X := 1, Y := 2);
var A: PointAlias := P; // Valid: PointAlias names the Point declaration.
var S: Size := P;

```

Two values of the same record type compare with `=` and `<>` field by field when every field
compares; see [Operators](../basics/operators.md). Array and dictionary fields compare when their
components compare. Resource, task or callable components prevent whole-value
equality, including when nested inside Option or Result.

Construction always names a declared record type. Obsolete `record Field := Value; end record` expressions are rejected. When an assignment, argument, collection
or return supplies a record target, the diagnostic reports its resolved declaration,
including through imported aliases. Without a resolved target it requests a declared
constructor rather than guessing a type from visible fields.

## Accessing fields

```pascal
var PosX: real := P.X;

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

A named record with at least one private field can be constructed only inside its declaring unit, even if all private fields have default
values. Importers obtain such values from public functions or static functions.
They may copy received values and use record updates for public fields; private
fields are preserved and cannot be named in an update.

Record member visibility is valid only for records declared in unit files.
Functions, procedures, properties, and events use the same private-default
rule.

## Immutability

Record instances follow the same immutability rules as variables. A `mutable var` record allows field reassignment:

```pascal
mutable var P: Point := Point(X := 1.0, Y := 2.0);

begin
  P.X := 10.0;  // Valid — P is mutable
end;
```

## Default field values

A field declaration may include a default value using `:=`. When a constructor omits a field that has a default, the compiler substitutes the default automatically. Fields without a default must always be supplied.

Defaults are expressions checked against the field type, including nested record
and collection values, calls and callable values. An omitted field evaluates its
default when constructing the record; an explicit value skips that default.
For public records imported from another unit, scalar constant defaults may use
arithmetic, comparisons, boolean operators and string concatenation, as well as
local constants and directly imported scalar constants. Their evaluated values
are stored in the compiled-unit interface.

Type aliases retain the declaring record's defaults, including when another unit
reexports an alias or a collection of that record type. Fields without defaults
remain required. An alias preserves the original field visibility and does not
permit construction outside the declaring unit when private fields exist.

```pascal
type Config = record
  Host: string := 'localhost';
  Port: integer := 8080;
  Debug: boolean := false;
end record;

```

Omitting defaulted fields:

```pascal
var C: Config := Config(); // Host='localhost', Port=8080, Debug=false
var D: Config := Config(Port := 9000);

```

Explicitly providing a value overrides the default:

```pascal
var E: Config := Config(Host := 'example.com', Port := 443, Debug := true);

```

Fields without a default remain required:

```pascal
type Vertex = record
  Id: integer; // Required
  X: integer := 0; // Optional
  Y: integer := 0; // Optional
end record;

var V: Vertex := Vertex(Id := 7);

```

## See also

- [Record methods](record-methods.md)
- [Visibility](../../program-structure/visibility.md)
- [Record properties](record-properties.md)
- [Record events](record-events.md)
- [Record update](record-update.md)
- [Mutable parameters](../functions/mutable-parameters.md)
