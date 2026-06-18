# `Std.Env`

Process environment access for hosted FPAS programs. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Env, Std.Option;
begin
  if Exists('PATH') then
    WriteLn(Std.Option.Unwrap(Get('PATH')))
end.
```

`Std.Env` reads the environment visible to the host process. It is UI-independent: console, TUI, graph, and background-task programs can import it when they need environment values.


## Importing and names

After `uses Std.Env;` use **`Get`**, **`Exists`**, or the fully qualified forms **`Std.Env.Get`**, **`Std.Env.Exists`**.

---

## Quick reference

Requires `uses Std.Env;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `Get(Name: string): Option of string` | returns `Some(Value)` when the variable exists, otherwise `None` |
| function | `Exists(Name: string): boolean` | checks whether the variable is present |

Environment lookup is process-wide and effectful because it reads host process state. `Std.Env` does not mutate environment variables.

---

## `function Get(Name: string): Option of string`

Returns the environment variable named `Name`, or `None` when it is missing.

```pascal
var Home: Option of string := Get('HOME');
if Std.Option.IsSome(Home) then
  WriteLn(Std.Option.Unwrap(Home))
```

---

## `function Exists(Name: string): boolean`

Returns `true` when the process environment contains `Name`.

```pascal
if Exists('PATH') then
  WriteLn('PATH is available')
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`env.rs`](../../../../crates/fpas-std/src/env.rs) |
| Call lowering | [`std_calls/env.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/env.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Intrinsic ids | [`intrinsic/env.rs`](../../../../crates/fpas-bytecode/src/intrinsic/env.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
