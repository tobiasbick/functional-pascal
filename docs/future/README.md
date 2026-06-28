# Future Features

Open planning items for Functional Pascal.

## Standard library roadmap

Implemented today: `Std.Console`, `Std.Args`, `Std.Env`, `Std.Proc`, `Std.Path`, `Std.Fs`, `Std.Time`, `Std.Graph`, `Std.Tui`, `Std.Str`, `Std.Conv`, `Std.Parse`, `Std.Json`, `Std.Math`, `Std.Random`, `Std.Array`, `Std.Dict`, `Std.Result`, `Std.Option`, `Std.Task`. Later candidates: [`std-roadmap.md`](std-roadmap.md).

## Projects and libraries

Source-level `library` projects and `[dependencies]` are **implemented** — the loader merges `.fpas` from dependency manifests. See [`libraries.md`](libraries.md) for export-API ideas and scope notes.

## Test framework

FPAS programs today are tested mainly from Rust (`compile_and_run`, VM input queues). Author-facing tests use `Std.Test`, `fpas test`, and optional sidecars — see [`docs/pascal/std/testing/test.md`](../pascal/std/testing/test.md). Planned runner docs: [`test-framework/README.md`](test-framework/README.md) (when present).

## Source formatter

[`fpas fmt`](../../crates/fpas-cli/src/cli_fmt/) is **implemented** ([`fpas-fmt`](../../crates/fpas-fmt/)). Manual CLI only — run when you want formatted output. Normative rules: [`docs/pascal/tools/fmt-style.md`](../pascal/tools/fmt-style.md).

## Documentation layout

Topic directories under [`docs/pascal/`](../pascal/) replaced the flat numbered chapters (`01-overview.md` … `11-stdlib.md`).

## Under Consideration

| # | Feature | Description |
|---|---------|-------------|
| 9 | [`dict`](09-remove-dict.md) | Pending — may be kept |
| — | [Task memory benchmark](task-memory-benchmark.md) | Reproduce pkolaczk async memory bench with `go` + `WaitAll`; blocked on cooperative sleep and scale validation |
| — | [Libraries — finer exports](libraries.md) | Per-symbol / re-export rules beyond `[exports].units` |

## Not Yet Planned
