# Units

The unit system enables multi-file projects. Each source file declares its namespace via a `unit` declaration. All project files are listed in the project `.fpasprj` file (see [Projects](projects.md)).

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`unit_decl`, `program_decl`, `uses_clause`).

## Unit declaration

A unit file starts with a `unit` declaration followed by declarations (functions, procedures, types, constants, `var`, and `mutable var`). There is no main block.

```pascal
unit MyApp.Utils;
uses Std.Str;

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
  return Length(Trim(S)) = 0
end;
```

## Program file

The program file uses a `program` declaration instead of `unit`. It does not define a namespace and is the entry point of the application. There is exactly one program file per project. See [Projects](projects.md) for project structure details.

## Using units

Units must be explicitly imported via `uses` to be accessible — including `Std.*` units. Being listed in the project `.fpasprj` file does not make a unit automatically visible.

```pascal
program Main;

uses
  MyApp.Utils,
  Std.Console;

begin
  var Clamped: integer := Clamp(150, 0, 100);
  WriteLn(Clamped);  { 100 }
end.
```

## Short names and qualified names

When a unit is imported via `uses`, its exported symbols become available by their short (unqualified) name and by their fully qualified name:

```pascal
program Hello;
uses Std.Console;
begin
  WriteLn('short');              { OK — short name }
  Std.Console.WriteLn('full');   { OK — fully qualified }
end.
```

### Ambiguity rule

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

## Reserved namespace `Std`

The first segment `Std` (ASCII, any case) is reserved for the standard library. User-defined units use another root segment (for example `MyApp.Utils`).

In `uses`, standard library entries are two-part names such as `Std.Console`.

Unknown `uses` entries referring to `Std.*` are rejected with an error that lists the valid standard units.

## Unit resolution

Units are resolved through the project `.fpasprj` file, which lists all source files belonging to the project. Each file declares its namespace via its `unit` declaration. The directory structure has no influence on the unit name — only the `unit` declaration inside the file matters.

Library units from other projects are merged in when the consuming `.fpasprj` lists them under `[dependencies].projects` (path to another manifest) or `[dependencies].workspace` (member `project.name` inside an enclosing `.fpasworkspace`). A library may restrict which units are importable via `[exports].units` in its `.fpasprj`. See [Projects](projects.md).

Only units reachable from the program file's `uses` chain (including transitive dependencies) are compiled into the final program.

## See also

- [Visibility](visibility.md)
- [Projects](projects.md)
- [Standard library](../std/README.md)
