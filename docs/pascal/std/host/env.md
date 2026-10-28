# `Std.Env`

Process environment access for hosted FPAS programs. This page is the full API for the unit.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Env as Env;

begin
  if Env.Exists('PATH') then
    Console.WriteLn(Env.Get('PATH').Unwrap());
  end if;
end.
```

`Std.Env` reads the environment visible to the host process. It is UI-independent: console, TUI, and background-task programs can import it when they need environment values.


## Importing and names

Import with `uses Std.Env as Env;` and access public symbols as `Env.Name`. User enum variants require `Env.Type.Variant`.

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

```pascal id=example-2 fragment context=body-console-env
const Home: option of string := Env.Get('HOME');
case Home of
  when Some(const Value):
    Console.WriteLn(Value);
  when None:
    null;
end case;
```

---

## `function Exists(Name: string): boolean`

Returns `true` when the process environment contains `Name`.

```pascal id=example-3 fragment context=body-console-env
if Env.Exists('PATH') then
  Console.WriteLn('PATH is available');
end if;
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`env.rs`](../../../../crates/fpas-runtime/src/stdlib/env.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Intrinsic ids | [`intrinsic/env.rs`](../../../../crates/fpas-code/src/intrinsic/env.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
