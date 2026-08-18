# Crate review findings (2026-08)

Read-only review of the Rust workspace crates (~180k LOC, 21 crates). No P0 (universal release blocker) was found. Residual risk is concentrated in semantic soundness, a few codegen holes, task-runtime composition, and Std/DAP/fmt paths that disagree with their own spec.

This file is the **defect list**. Implement from [`tasks/`](tasks/), not by scanning this page and improvising.

Healthy areas (no demonstrated soundness hole in this pass): `fpas-ir`, `fpas-unit`, `fpas-program`, `fpas-bytecode`, `fpas-bundle`, `fpas-build` sidecar reuse, `fpas-diagnostics` rendering. Interpreter bounds/division/type errors generally become diagnostics. LSP UTF-16 conversion is explicit and tested. Formatter corpus round-trip exists; the holes are cases the corpus does not include.

## P1

| Area | Title | Location |
|------|-------|----------|
| Sema | `GenericParam` is compatible with every type | `crates/fpas-sema/src/types.rs:267` |
| Sema | Named public/program records compared by field shape, not identity | `crates/fpas-sema/src/types.rs:280` |
| Sema | Record-literal defaults only applied on `var` initializers | `crates/fpas-sema/src/check/decl/vars.rs:16` |
| Sema | Imported `Color.Red` stays bound to the first install when two `Color` types clash | `crates/fpas-sema/src/interface/install.rs:144` |
| Compiler | Alias/import-qualified simple enums lower as variant index, not backing integer | `crates/fpas-compiler/src/lowering/expr/designators.rs:88` |
| Compiler | Closures in program/unit `var` initializers are never registered (ICE) | `crates/fpas-compiler/src/lowering/mod.rs:180` |
| VM | `Wait` / `WaitAll` hot-spin when shutdown is set and the handle is still `Pending` | `crates/fpas-vm/src/vm/tasks/mod.rs:180` |
| VM | Inline-helped task panic is attributed to the waiter | `crates/fpas-vm/src/vm/tasks/pool.rs:40` |
| VM | Hosted callbacks inherit owner `task_id`; `Sleep`/`Yield` suspends the owner | `crates/fpas-vm/src/vm/callback_call.rs:100` |
| Std | `Slice` / `Substring` / `Delete` overflow `start + len` instead of a runtime error | `crates/fpas-std/src/array.rs:95` |
| Std | `FromChar` errors on `Count < 0` and has no length cap | `crates/fpas-std/src/str.rs:189` |
| Std | `Window` / `GotoXY` swallow out-of-range coordinates | `crates/fpas-std/src/console/operations/window.rs:67` |
| Debug | Blank JSONL line terminates the session; `--commands` can exit 0 | `crates/fpas-debug/src/jsonl/server.rs:140` |
| Debug | Failed DAP `setBreakpoints` leaks already-installed breakpoints | `crates/fpas-debug/src/dap/server/breakpoints.rs:26` |
| Debug | DAP source match is case-sensitive and misses remapped library paths | `crates/fpas-debug/src/dap/server/breakpoints.rs:122` |
| Debug | DAP `levels`/`count` `0` means empty, not “all” | `crates/fpas-debug/src/dap/server/dispatch.rs:67` |
| CLI | `--timeout` does not cover isolated-worker startup before `ready` | `crates/fpas-cli/src/cli_test/process/mod.rs:255` |
| Lexer | `1.0e_3` lexes as `1000.0` with no diagnostic | `crates/fpas-lexer/src/lexer/numbers.rs:107` |
| Parser | Missing-expression recovery consumes `end` / `)` / `then` / … | `crates/fpas-parser/src/parser/expr/primary.rs:101` |
| Project | Shared library+consumer file is retagged `Own` (export bypass or false reject) | `crates/fpas-project/src/dependencies.rs:187` |
| Fmt | `//` comments on `uses` items are dropped | `crates/fpas-fmt/src/comments/traversal.rs:64` |

## P2

| Area | Title | Location |
|------|-------|----------|
| Sema | Enum exhaustiveness uses the last identifier, so a shadowed `Red` counts as covered | `crates/fpas-sema/src/check/stmt/control_flow/if_case/exhaustiveness.rs:22` |
| Sema | Public signature may name a private type with no export diagnostic | `crates/fpas-sema/src/interface/export.rs:39` |
| VM | Failed `IndexSet` leaves the destination slot empty | `crates/fpas-vm/src/vm/execute/aggregates.rs:102` |
| VM | `Application.Run` error leaves `run_active` set | `crates/fpas-vm/src/vm/hosted/graph/host.rs:264` |
| Std | `Pad*` / `IntToHex` accept unbounded widths | `crates/fpas-std/src/str.rs:349` |
| Std | `ScreenWidth`/`Height` can disagree with the retained screen after resize | `crates/fpas-std/src/console/operations/io.rs:108` |
| Parser | Empty `const` / `var` / `type` / `enum` / `case` bodies parse with no error | `crates/fpas-parser/src/parser/decl/mod.rs:11` |
| Parser | Invalid top-level `static` discards the recovered routine AST | `crates/fpas-parser/src/parser/decl/mod.rs:49` |
| Linker | Record/enum coalescing ignores field types | `crates/fpas-linker/src/symbols.rs:161` |
| Project | POSIX-style absolute paths are not treated as absolute on Windows | `crates/fpas-project/src/paths.rs:215` |
| LSP | Unreadable sibling project file fail-closes analysis (stale diagnostics) | `crates/fpas-language-service/src/analysis/mod.rs:250` |
| LSP | Discovery can walk above the workspace when `starts_with` fails (Windows case) | `crates/fpas-language-service/src/workspace/discovery.rs:34` |
| LSP | `lexical_normalize` pops `ParentDir` with no root guard | `crates/fpas-language-service/src/document/mod.rs:33` |
| Fmt | Unprintable string bytes are written inside quotes instead of `#` codes | `crates/fpas-fmt/src/emit/expr/literal.rs:114` |
| CLI | `fpas test` has no default timeout | `crates/fpas-cli/src/cli_input/mod.rs:233` |
| Debug | DAP/JSONL `read_line` for headers/lines is uncapped | `crates/fpas-debug/src/dap/framing.rs:20` |
| CLI | `fpas fmt` globs follow symlinks and write through them | `crates/fpas-cli/src/cli_fmt/paths.rs:141` |

## P3

| Area | Title | Location |
|------|-------|----------|
| CLI | `--report jsonl` is parsed and documented, then discarded | `crates/fpas-cli/src/cli_input/options.rs:243` |
| Project | No test that program/test manifests cannot appear in `[dependencies]` | `crates/fpas-project/src/dependencies.rs:167` |
