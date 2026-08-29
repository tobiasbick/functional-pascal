# Records

Records group related data together.

Records may refer to themselves through an aggregate field such as
`Children: array of Element`. These recursive record types are valid in ordinary variables,
function return types, first-class function signatures, and generic routine inference.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_type`, record literals in expressions).

## Declaring a record

```pascal
type
  Point = record
    X: real;
    Y: real;
  end;
```

## Creating a record

```pascal
var
  P: Point := record
    X := 0.0;
    Y := 5.0;
  end;
```

Each field may appear at most once in a record literal. Field names are
case-insensitive, so `X` and `x` identify the same field and cannot both be
specified.

## Type identity and compatibility

Each named record declaration defines a distinct type. Two records are not compatible merely
because they contain fields with the same names and types. Assignments, arguments, and return
values must use the same record declaration or an alias of that declaration.

```pascal
type
  Point = record
    X: integer;
    Y: integer;
  end;

  Size = record
    X: integer;
    Y: integer;
  end;

  PointAlias = Point;

var
  P: Point := record
    X := 1;
    Y := 2;
  end;
  A: PointAlias := P;  // Valid: PointAlias names the Point declaration.
  S: Size := P;        // Error: Point and Size are distinct declarations.
```

An anonymous record literal receives the expected named record type from its assignment, argument,
array element, constant, or return context. This keeps direct construction concise without making
separately declared record types interchangeable.

## Accessing fields

```pascal
var
  PosX: real := P.X;
```

## Field visibility

Fields in records declared by a unit are private by default. Write `public`
directly before each field that importing units may access. FPAS has no
visibility sections and no explicit `private` keyword.

```pascal
unit MyApp.Counters;

public type
  Counter = record
    Value: integer;
    public Step: integer;
  end;
```

Code in `MyApp.Counters` may read and write `Value`. Importing units may use
`Step`, but cannot name `Value`. An explicit `public` modifier has the same
meaning for that field.

A named record with at least one private field can be constructed with a record
literal only inside its declaring unit, even if all private fields have default
values. Importers obtain such values from public functions or static functions.
They may copy received values and use record updates for public fields; private
fields are preserved and cannot be named in an update.

Record member visibility is valid only for records declared in unit files.
Functions, procedures, properties, and events use the same private-default
rule.

## Immutability

Record instances follow the same immutability rules as variables. A `mutable var` record allows field reassignment:

```pascal
mutable var
  P: Point := record
    X := 1.0;
    Y := 2.0;
  end;

begin
  P.X := 10.0;  // Valid — P is mutable
end.
```

## Default field values

A field declaration may include a default value using `:=`. When a record literal omits a field that has a default, the compiler substitutes the default automatically. Fields without a default must always be supplied.

```pascal
type
  Config = record
    Host: string := 'localhost';
    Port: integer := 8080;
    Debug: boolean := false;
  end;
```

Omitting defaulted fields:

```pascal
var
  C: Config := record end;                 // Host='localhost', Port=8080, Debug=false
  D: Config := record Port := 9000; end;   // Host='localhost', Port=9000, Debug=false
```

Explicitly providing a value overrides the default:

```pascal
var
  E: Config := record Host := 'example.com'; Port := 443; Debug := true; end;
```

Fields without a default remain required:

```pascal
type
  Vertex = record
    Id: integer;           // Required
    X: integer := 0;       // Optional
    Y: integer := 0;       // Optional
  end;

var
  V: Vertex := record Id := 7; end;  // X=0, Y=0 from defaults
```

## See also

- [Record methods](record-methods.md)
- [Visibility](../../program-structure/visibility.md)
- [Record properties](record-properties.md)
- [Record events](record-events.md)
- [Record update](record-update.md)
- [Mutable parameters](../functions/mutable-parameters.md)
