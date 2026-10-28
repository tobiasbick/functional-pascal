# `Std.Os.Args`

Process argument access for hosted FPAS programs. This page is the full API for the unit.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Os.Args as Args;

begin
  Console.WriteLn(Args.ParamCount());
  if Args.ParamCount() > 0 then
    Console.WriteLn(Args.ParamStr(0));
  end if;
end.
```

Run program arguments after the CLI separator:

```text
fpas run app.fpas -- input.txt verbose
```


## Importing and names

Import with `uses Std.Os.Args as Args;` and access public symbols as `Args.Name`. User enum variants require `Args.Type.Variant`.

---

## Quick reference

Requires `uses Std.Os.Args as Args;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `ParamCount(): integer` | number of program arguments after `--` |
| function | `ParamStr(Index: integer): string` | 0-based argument lookup |

The input file name and the `fpas` executable name are not included. Only values after `--` are visible.

---

## `function ParamCount(): integer`

Returns the number of program arguments supplied after the CLI separator.

```pascal id=example-3 fragment context=body-console-args
Console.WriteLn(Args.ParamCount());
```

---

## `function ParamStr(Index: integer): string`

Returns the argument at 0-based `Index`.

Runtime error if `Index` is negative or greater than or equal to `ParamCount()`.

```pascal id=example-4 fragment context=body-console-args
if Args.ParamCount() > 0 then
  Console.WriteLn(Args.ParamStr(0));
end if;
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| VM execution | [`args.rs`](../../../../crates/fpas-runtime/src/execution/vm/hosted/args.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Intrinsic ids | [`intrinsic/args.rs`](../../../../crates/fpas-code/src/intrinsic/args.rs) |

## See also

- [Host I/O index](README.md)
- [`Std.Cli`: typed argument schemas and generated help](cli.md)
- [Standard library index](../README.md)
