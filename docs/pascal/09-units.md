# 9. Units

The unit system enables multi-file projects. Each source file declares its namespace via a `unit` declaration. All project files are listed in the project `.fpasprj` file (see [10-projects.md](10-projects.md)).

## Unit Declaration

A unit file starts with a `unit` declaration followed by declarations (functions, procedures, types, constants). There is no main block.

```pascal
unit MyApp.Utils;

function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min
  else if Value > Max then
    return Max
  else
    return Value
end;

function IsBlank(S: string): boolean;
begin
  return Std.Str.Length(Std.Str.Trim(S)) = 0
end;
```

## Program File

The program file uses a `program` declaration instead of `unit`. It does not define a namespace and is the entry point of the application. There is exactly one program file per project. See [10-projects.md](10-projects.md) for project structure details.

## Using Units

Units must be explicitly imported via `uses` to be accessible — including `Std.*` units. Being listed in the project `.fpasprj` file does not make a unit automatically visible.

```pascal
program Main;

uses
  MyApp.Utils,
  Std.Console;

begin
  var Clamped: integer := MyApp.Utils.Clamp(150, 0, 100);
  Std.Console.WriteLn(Clamped);  { 100 }
end.
```

## Short Names and Qualified Names

When a unit is imported via `uses`, its exported symbols become available by their short (unqualified) name and by their fully qualified name:

```pascal
program Hello;
uses Std.Console;
begin
  WriteLn('short');              { OK — short name }
  Std.Console.WriteLn('full');   { OK — fully qualified }
end.
```

### Ambiguity Rule

When two or more imported units export the same short name, the short name becomes ambiguous. No error is raised at the `uses` site; the compiler reports an error only when the ambiguous short name is actually used. The fully qualified name always works as a fallback:

```pascal
program Demo;
uses Std.Str, Std.Array;           { OK — no error at import }
begin
  { Length('hi');   ← ERROR: ambiguous — exists in Std.Str and Std.Array }
  var L1: integer := Std.Str.Length('hi');       { OK }
  var L2: integer := Std.Array.Length([1, 2]);   { OK }
end.
```

## Visibility

All declarations in a unit are **public by default**. Use the `private` keyword to restrict a declaration to the unit that defines it.

| Annotation | Meaning |
|---|---|
| *(none)* | Public (default) — visible to importers |
| `public` | Public (explicit, optional) — same as default |
| `private` | Unit-internal — not visible to importers |

`private` declarations are compiled and available within the unit but are excluded from the unit's export table. Other units cannot reference them by short name or qualified name.

```pascal
unit MyApp.Geometry;
uses Std.Math;

type
  Point = record
    X: real;
    Y: real;
  end;

function Distance(A: Point; B: Point): real;
begin
  return Std.Math.Sqrt(Square(B.X - A.X) + Square(B.Y - A.Y))
end;

private function Square(V: real): real;
begin
  return V * V
end;
```

`Point` and `Distance` are public. `Square` is private — only callable from within `MyApp.Geometry`.

The `private` and `public` keywords apply to `function`, `procedure`, `type`, `const`, and `var` declarations in units. They are not valid in `program` files (program-level declarations have no importers).

## Reserved Namespace `Std`

The first segment `Std` (ASCII, any case) is reserved for the standard library. User-defined units must use another root segment (for example `MyApp.Utils`, not `Std.Utils`). Only the compiler and bundled libraries may define names under `Std.*`.

Unknown `uses` entries referring to `Std.*` are rejected with an error that lists the valid standard units.

## Unit Resolution

Units are resolved through the project `.fpasprj` file, which lists all source files belonging to the project. Each file declares its namespace via its `unit` declaration. The directory structure has no influence on the unit name — only the `unit` declaration inside the file matters.

A unit that is listed in the project file but not imported via `uses` is not compiled into the final program. Only units reachable from the program file's `uses` chain (including transitive dependencies) are included.
