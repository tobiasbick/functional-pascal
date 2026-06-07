# Future Features

Open planning items for Functional Pascal.

## VM implementation (Rust)

[`parallel-vm.md`](../rust/parallel-vm.md) documents the **implemented** parallel task runtime layout in `fpas-vm` (bytecode through shutdown): summary by integration step, paths, and tests. Language rules stay in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md).

## TUI roadmap

Hosted terminal UI is **implemented**: Rust-owned event loop, FPAS `On*` handlers, host widgets (menu bar, status bar, solid fill), and `Application.Run` as the primary entry for full apps. Low-level terminal events (`ReadEvent`, `PollEvent`) live on **`Std.Console`** for CRT-style scripts; they are **not** part of **`Std.Tui`**. Implementation history and remaining ideas: [`tui-application-framework.md`](tui-application-framework.md). User-facing spec: [`docs/pascal/std/tui-app.md`](../pascal/std/tui-app.md).

## Standard library roadmap

Implemented today: `Std.Console`, `Std.Args`, `Std.Env`, `Std.Proc`, `Std.Path`, `Std.Fs`, `Std.Time`, `Std.Graph`, `Std.Tui`, `Std.Str`, `Std.Conv`, `Std.Parse`, `Std.Json`, `Std.Math`, `Std.Random`, `Std.Array`, `Std.Dict`, `Std.Result`, `Std.Option`, `Std.Task`. Later candidates: [`std-roadmap.md`](std-roadmap.md).

## Projects and libraries

Source-level `library` projects and `[dependencies]` are **implemented**. Precompiled artifacts are **not** planned. See [`libraries.md`](libraries.md) for scope and possible export-API work.

## Under Consideration

| # | Feature | Description |
|---|---------|-------------|
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |
| — | [Libraries — finer exports](libraries.md) | Per-symbol / re-export rules beyond `[exports].units` |

## Not Yet Planned
