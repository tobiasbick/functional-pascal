# `Std.Args`

Process argument access for hosted FPAS programs. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Args;
begin
  WriteLn(ParamCount());
  if ParamCount() > 0 then
    WriteLn(ParamStr(0))
end.
```

Run program arguments after the CLI separator:

```text
fpas app.fpas -- input.txt verbose
```

**Maintenance (implementers only):** align with [`std_registry/`](../../../../crates/fpas-sema/src/std_registry/mod.rs), [`std_calls/args.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/args.rs), [`vm/execute/io/args.rs`](../../../../crates/fpas-vm/src/vm/execute/io/args.rs), and [`intrinsic/args.rs`](../../../../crates/fpas-bytecode/src/intrinsic/args.rs).

---

## Importing and names

After `uses Std.Args;` use **`ParamCount`**, **`ParamStr`**, or the fully qualified forms **`Std.Args.ParamCount`**, **`Std.Args.ParamStr`**.

---

## Quick reference

Requires `uses Std.Args;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `ParamCount(): integer` | number of program arguments after `--` |
| function | `ParamStr(Index: integer): string` | 0-based argument lookup |

The input file name and the `fpas` executable name are not included. Only values after `--` are visible.

---

## `function ParamCount(): integer`

Returns the number of program arguments supplied after the CLI separator.

```pascal
WriteLn(ParamCount())
```

---

## `function ParamStr(Index: integer): string`

Returns the argument at 0-based `Index`.

Runtime error if `Index` is negative or greater than or equal to `ParamCount()`.

```pascal
if ParamCount() > 0 then
  WriteLn(ParamStr(0))
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| VM execution | [`args.rs`](../../../../crates/fpas-vm/src/vm/execute/io/args.rs) |
| Call lowering | [`std_calls/args.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/args.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Intrinsic ids | [`intrinsic/args.rs`](../../../../crates/fpas-bytecode/src/intrinsic/args.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
