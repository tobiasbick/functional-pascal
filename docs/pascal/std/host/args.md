# `Std.Args`

Process argument access for hosted FPAS programs. This page is the full API for the unit.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Args as Args;

begin
  Console.WriteLn(Args.ParamCount());
  if Args.ParamCount() > 0 then
    Console.WriteLn(Args.ParamStr(0));
  end if;
end program;
```

Run program arguments after the CLI separator:

```text
fpas run app.fpas -- input.txt verbose
```


## Importing and names

Import with `uses Std.Args as Args;`. Access every exported member through `Args`, for example `Args.ParamStr(...)`. Imports open no short names.

---

## Quick reference

Requires `uses Std.Args as Args;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `ParamCount(): integer` | number of program arguments after `--` |
| function | `ParamStr(Index: integer): string` | 0-based argument lookup |

The input file name and the `fpas` executable name are not included. Only values after `--` are visible.

---

## `function ParamCount(): integer`

Returns the number of program arguments supplied after the CLI separator.

```pascal
uses Std.Console as Console;
uses Std.Args as Args;

Console.WriteLn(Args.ParamCount());
```

---

## `function ParamStr(Index: integer): string`

Returns the argument at 0-based `Index`.

Runtime error if `Index` is negative or greater than or equal to `ParamCount()`.

```pascal
uses Std.Console as Console;
uses Std.Args as Args;

if Args.ParamCount() > 0 then
  Console.WriteLn(Args.ParamStr(0));
end if;
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| VM execution | [`args.rs`](../../../../crates/fpas-vm/src/vm/hosted/args.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Intrinsic ids | [`intrinsic/args.rs`](../../../../crates/fpas-bytecode/src/intrinsic/args.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
