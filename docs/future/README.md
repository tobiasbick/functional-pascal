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

FPAS programs today are tested mainly from Rust (`compile_and_run`, VM input queues). Author-facing tests use `Std.Test`, `fpas test`, and scripted keyboard/mouse sidecars — see [`docs/pascal/std/test.md`](../pascal/std/test.md). Planned runner docs: [`test-framework/README.md`](test-framework/README.md) (when present).

**Under consideration:** two complementary directions for richer TUI testing —
- In-process: test the hosted TUI fully from FPAS via `fpas test` (headless pump, input injection, screen/view/widget introspection). Proposal: [`tui-tests-fpas/README.md`](tui-tests-fpas/README.md).
- Out-of-process: a localhost HTTP control server for live inspection and event injection (Playwright-style, no PTY). Proposal: [`tui-test/README.md`](tui-test/README.md) (API: [`control-server-api.md`](tui-test/control-server-api.md), architecture: [`architecture.md`](tui-test/architecture.md)).

## Source formatter

[`fpas fmt`](../../crates/fpas-cli/src/cli_fmt/) is **implemented** ([`fpas-fmt`](../../crates/fpas-fmt/)). Manual CLI only — run when you want formatted output. Normative rules: [`docs/rust/fmt-style.md`](../rust/fmt-style.md).

## Under Consideration

| # | Feature | Description |
|---|---------|-------------|
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |
| — | [Native TUI testing in FPAS](tui-tests-fpas/README.md) | Test the hosted TUI fully from `fpas test`: headless pump, input injection, screen/view/widget introspection |
| — | [TUI live control server](tui-test/README.md) | Localhost HTTP API to query screen/views and inject events during `Application.Run` |
| — | [Task memory benchmark](task-memory-benchmark.md) | Reproduce pkolaczk async memory bench with `go` + `WaitAll`; blocked on cooperative sleep and scale validation |
| — | [Libraries — finer exports](libraries.md) | Per-symbol / re-export rules beyond `[exports].units` |

## Not Yet Planned
