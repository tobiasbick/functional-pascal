# Records

Records group related data together.

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
- [Record properties](record-properties.md)
- [Record events](record-events.md)
- [Record update](record-update.md)
- [Mutable parameters](../functions/mutable-parameters.md)
