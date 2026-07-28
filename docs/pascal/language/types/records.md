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

## Accessing fields

```pascal
var
  PosX: real := P.X;
```

## Field visibility

Fields in records declared by a unit are public by default. Write `private` or
`public` directly before an individual field; FPAS has no visibility sections.

```pascal
unit MyApp.Counters;

type
  Counter = record
    private Value: integer;
    Step: integer;
  end;
```

Code in `MyApp.Counters` may read and write `Value`. Importing units may use
`Step`, but cannot name `Value`. An explicit `public` modifier has the same
meaning as the public default.

A named record with at least one private field can be constructed with a record
literal only inside its declaring unit, even if all private fields have default
values. Importers obtain such values from public functions or static functions.
They may copy received values and use record updates for public fields; private
fields are preserved and cannot be named in an update.

Record field visibility is valid only for records declared in unit files.
Properties and events remain public.

## Immutability

Record instances follow the same immutability rules as variables. A `mutable var` record allows field reassignment:

```pascal
mutable var
  P: Point := record
    X := 1.0;
    Y := 2.0;
  end;

begin
  P.X := 10.0;  { OK — P is mutable }
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
  C: Config := record end;          { Host='localhost', Port=8080, Debug=false }
  D: Config := record Port := 9000; end;  { Host='localhost', Port=9000, Debug=false }
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
    Id: integer;           { required }
    X: integer := 0;       { optional }
    Y: integer := 0;       { optional }
  end;

var
  V: Vertex := record Id := 7; end;  { X=0, Y=0 from defaults }
```

## See also

- [Record methods](record-methods.md)
- [Visibility](../../program-structure/visibility.md)
- [Record properties](record-properties.md)
- [Record events](record-events.md)
- [Record update](record-update.md)
- [Mutable parameters](../functions/mutable-parameters.md)
