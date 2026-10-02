# `Std.Env`

Process environment access for hosted FPAS programs. This page is the full API for the unit.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Env as Env;
uses Std.Options as Options;

begin
  if Env.Exists('PATH') then
    Console.WriteLn(Options.Unwrap(Env.Get('PATH')));
  end if;
end program;
```

`Std.Env` reads the environment visible to the host process. It is UI-independent: console, TUI, and background-task programs can import it when they need environment values.


## Importing and names

Import with `uses Std.Env as Env;`. Access every exported member through `Env`, for example `Env.Get(...)`. Imports open no short names.

---

## Quick reference

Requires `uses Std.Env as Env;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `Get(Name: string): Option of string` | returns `Some(Value)` when the variable exists, otherwise `None` |
| function | `Exists(Name: string): boolean` | checks whether the variable is present |

Environment lookup is process-wide and effectful because it reads host process state. `Std.Env` does not mutate environment variables.

---

## `function Get(Name: string): Option of string`

Returns the environment variable named `Name`, or `None` when it is missing.

```pascal
uses Std.Console as Console;
uses Std.Env as Env;
uses Std.Options as Options;

var Home: option of string := Env.Get('HOME');
if Options.IsSome(Home) then
  Console.WriteLn(Options.Unwrap(Home));
end if;
```

---

## `function Exists(Name: string): boolean`

Returns `true` when the process environment contains `Name`.

```pascal
uses Std.Console as Console;
uses Std.Env as Env;

if Env.Exists('PATH') then
  Console.WriteLn('PATH is available');
end if;
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`env.rs`](../../../../crates/fpas-std/src/env.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Intrinsic ids | [`intrinsic/env.rs`](../../../../crates/fpas-bytecode/src/intrinsic/env.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
