# Future Features

Open planning items for Functional Pascal.

## VM implementation (Rust)

[`parallel-vm.md`](../rust/parallel-vm.md) documents the **implemented** parallel task runtime layout in `fpas-vm` (bytecode through shutdown): summary by integration step, paths, and tests. Language rules stay in [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md).

## TUI roadmap

Hosted terminal UI is **implemented**: Rust-owned event loop, FPAS `On*` handlers, host widgets (menu bar, status bar, solid fill), and `Application.Run` as the primary entry for full apps. Low-level terminal events (`ReadEvent`, `PollEvent`) live on **`Std.Console`** for CRT-style scripts; they are **not** part of **`Std.Tui`**. Reference implementations: [`apps/ide/`](../../apps/ide/) (multi-unit shell) and [`examples/pascal/tui/`](../../examples/pascal/tui/) (including [`menu_bar.fpas`](../../examples/pascal/tui/menu_bar.fpas)). Implementation history and remaining ideas: [`tui-application-framework.md`](tui-application-framework.md). User-facing spec: [`docs/pascal/std/tui-app.md`](../pascal/std/tui-app.md).

## Standard library roadmap

Implemented today: `Std.Console`, `Std.Args`, `Std.Env`, `Std.Proc`, `Std.Path`, `Std.Fs`, `Std.Time`, `Std.Graph`, `Std.Tui`, `Std.Str`, `Std.Conv`, `Std.Parse`, `Std.Json`, `Std.Math`, `Std.Random`, `Std.Array`, `Std.Dict`, `Std.Result`, `Std.Option`, `Std.Task`. Later candidates: [`std-roadmap.md`](std-roadmap.md).

## Projects and libraries

Source-level `library` projects and `[dependencies]` are **implemented**. Precompiled artifacts are **not** planned. See [`libraries.md`](libraries.md) for scope and possible export-API work.

## Test framework

FPAS programs today are tested mainly from Rust (`compile_and_run`, VM input queues). A JUnit-style author-facing layer — `Std.Test`, `fpas test`, scripted keyboard/mouse input — is **planned**. See [`test-framework/README.md`](test-framework/README.md) (implementation tasks: [`implementation.md`](test-framework/implementation.md); API draft: [`std-test.md`](test-framework/std-test.md), runner: [`runner.md`](test-framework/runner.md), sidecar format: [`scripted-input.md`](test-framework/scripted-input.md)).

## Source formatter

AST-based `fpas fmt` is **in progress**: [`fpas-fmt`](../../crates/fpas-fmt/) through declaration emitters (Phase 4); compilation units and CLI pending. Plan: [`formater/README.md`](formater/README.md) (style: [`formater/style.md`](formater/style.md); CLI TBD: [`formater/cli.md`](formater/cli.md)).

## Under Consideration

| # | Feature | Description |
|---|---------|-------------|
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |
| — | [Libraries — finer exports](libraries.md) | Per-symbol / re-export rules beyond `[exports].units` |

## Not Yet Planned
