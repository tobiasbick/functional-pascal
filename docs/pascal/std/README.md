# Standard library reference (`Std.*`)

This directory is the index for the Functional Pascal standard units.
Each unit page is written as a **self-contained handbook**: importing and short vs qualified names, a **quick reference** table, then **every** routine (and types where applicable) with parameters, behavior, edge cases, and a **small example**.

All units are opt-in through `uses`.

## Unit pages

- `Std.Console` - [console.md](console.md) (text I/O, CRT-style screen control, RGB/256 text colors, `ReadKey` / `KeyPressed`, `ReadKeyEvent`, `ReadEvent`, `KeyKind`, `KeyEvent`)
- `Std.Tui` - [tui.md](tui.md) (terminal application handle, size, key/resize events, redraw coordination)
- `Std.Str` - [str.md](str.md)
- `Std.Conv` - [conv.md](conv.md)
- `Std.Math` - [math.md](math.md)
- `Std.Array` - [array.md](array.md)
- `Std.Dict` - [dict.md](dict.md) (dictionaries — `Length`, `ContainsKey`, `Keys`, `Values`, `Remove`, `Get`, `Merge`)
- `Std.Result` - [result.md](result.md) (helpers for `Result of T, E` — `Unwrap`, `UnwrapOr`, `IsOk`, `IsError`, `Map`, `AndThen`, `OrElse`)
- `Std.Option` - [option.md](option.md) (helpers for `Option of T` — `Unwrap`, `UnwrapOr`, `IsSome`, `IsNone`, `Map`, `AndThen`, `OrElse`)

## Shared implementation touchpoints

When changing a `Std.*` API, update both docs and implementation wiring.
These files are the usual integration points:

- Intrinsic opcodes: [`crates/fpas-bytecode/src/intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs)
- Intrinsic dispatch (non-console): [`crates/fpas-std/src/intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs)
- Pascal types and `uses` registration: [`crates/fpas-sema/src/std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs)
