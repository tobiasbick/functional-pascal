# Verified crate-review findings (2026-08)

This is the evidence index for the current checkout. Implement only from the linked task files.
Source symbols are named instead of volatile line numbers.

No P0 release blocker was demonstrated. The review covered all 21 Rust workspace crates
(approximately 182,000 Rust lines including tests). Healthy areas in this pass include `fpas-ir`,
`fpas-unit`, `fpas-program`, `fpas-bytecode`, `fpas-bundle`, build-sidecar reuse, and diagnostic
rendering. “Healthy” means no defect was demonstrated by this review, not that the area is proven
bug-free.

## Confirmed P1 defects

| Area | Finding | Evidence | Task |
|---|---|---|---|
| Lexer | A leading underscore in an exponent is consumed as a separator | `fpas-lexer/lexer/numbers.rs::consume_digits_with` | [01](tasks/01-lexer-exponent.md) |
| Std | `Slice`, `Substring`, and `Delete` perform unchecked `start + len` | `fpas-std/src/array.rs`; `fpas-std/src/str.rs` | [02](tasks/02-std-range-overflow.md) |
| Std | `FromChar` contradicts documented non-positive behavior and allocation caps are missing | `fpas-std/src/str.rs`; `fpas-std/src/conv.rs` | [03](tasks/03-std-fromchar-pad.md) |
| Std | `Window` and `GotoXY` return success for invalid coordinates | `console/operations/window.rs`; `cursor.rs` | [04](tasks/04-std-console-coords.md) |
| Fmt | Comments anchored to individual `uses` entries are never emitted | `fpas-fmt/src/emit/program.rs::emit_optional_uses` | [05](tasks/05-fmt-uses-comments.md) |
| Debug/CLI | Fatal JSONL protocol input terminates the session but returns process success | `jsonl/server.rs::fatal_request`; `jsonl/transport.rs` | [06](tasks/06-debug-jsonl-fatal-exit.md) |
| Debug | DAP omitted/zero pagination is converted to finite or empty pages | `dap/server/dispatch.rs` | [07](tasks/07-dap-pagination.md) |
| Parser | Missing-expression recovery consumes parent synchronization tokens | `parser/expr/primary.rs::parse_primary_atom` | [09](tasks/09-parser-expr-recovery.md) |
| Compiler | Alias/import-qualified simple enums fall back to variant index | `lowering/expr/designators.rs::lower_designator_expression` | [11](tasks/11-compiler-enum-alias.md) |
| Compiler | Declaration initializers are absent from closure discovery | `lowering/mod.rs::lower_declarations_and_body` | [12](tasks/12-compiler-closure-init.md) |
| Sema | Ordinary body checks treat `GenericParam` as compatible with every type | `fpas-sema/src/types.rs::Ty::compatible_with` | [13](tasks/13-sema-generic-compat.md) |
| Sema | Record defaults/context are applied only in selected initializer paths | `check/decl/vars.rs::try_annotate_expected_record_literals` and callers | [15](tasks/15-sema-record-literals.md) |
| Sema | A second imported `Color` does not make the existing `Color.Red` binding ambiguous | `interface/install.rs::install_imported_enum_variants` | [16](tasks/16-sema-imported-enum.md) |
| VM | Pending `Wait`/`WaitAll` can spin after scheduler shutdown | `vm/tasks/mod.rs`; `tasks/scheduler.rs::wait_for_*` | [19](tasks/19-vm-wait-shutdown.md) |
| VM | Inline-helped task failure bypasses pool failure bookkeeping | `vm/tasks/pool.rs::run_helped` | [20](tasks/20-vm-help-fail.md) |
| Debug | Failed DAP source-breakpoint replacement can leak newly installed breakpoints | `dap/server/breakpoints.rs::set_source_breakpoints` | [27](tasks/27-dap-breakpoints.md) |
| Debug | DAP source resolution is case-sensitive and retains only portable aliases | `dap/server/breakpoints.rs::resolve_source_path`; `fpas-cli/src/cli_debug.rs::portable_path` | [28](tasks/28-dap-source-paths.md) |

## Confirmed P2 defects

| Area | Finding | Evidence | Task |
|---|---|---|---|
| Parser | Empty one-or-more declaration/enum/case bodies produce no parser diagnostic | `parser/decl/data/const_var.rs`; `type_defs.rs`; `parser/stmt/branching.rs` | [08](tasks/08-parser-empty-sections.md) |
| Parser | Invalid top-level `static` parses then discards the recovered routine AST | `parser/decl/mod.rs::recover_invalid_static_decl` | [10](tasks/10-parser-static-ast.md) |
| Sema | Enum exhaustiveness counts the raw last identifier instead of the resolved label kind | `if_case/exhaustiveness.rs::enum_label_name` | [17](tasks/17-sema-exhaustiveness.md) |
| VM | Failed `IndexSet` takes the destination value without restoring it | `vm/execute/aggregates.rs` | [22](tasks/22-vm-indexset-restore.md) |
| VM | Failed `Application.Run` can bypass graph-session cleanup | `vm/hosted/graph/host.rs` | [32](tasks/32-vm-graph-run-cleanup.md) |
| Linker | Duplicate layout coalescing compares names but not structural field types or members | `fpas-linker/src/symbols.rs::matching_layout_definition` | [24](tasks/24-linker-layouts.md) |
| LSP | A sibling snapshot I/O error prevents a current diagnostics publication | `language-service/analysis/mod.rs::project_snapshots`; `fpas-lsp/diagnostics/publisher.rs` | [29](tasks/29-lsp-sibling-io.md) |
| LSP | Case-only Windows path differences can disable the workspace-root discovery bound | `workspace/discovery.rs::discover_source_context` | [33](tasks/33-lsp-discovery-boundary.md) |
| Fmt | Parsed control characters are emitted literally inside quoted strings | `fpas-fmt/src/emit/expr/literal.rs::format_string` | [30](tasks/30-fmt-char-codes.md) |
| CLI | Glob expansion follows symlinked files and can write through them | `fpas-cli/src/cli_fmt/paths.rs::expand_glob` | [34](tasks/34-fmt-glob-symlinks.md) |
| Debug | DAP header lines/count and JSONL request lines are unbounded before parsing | `dap/framing.rs`; `jsonl/transport.rs` | [36](tasks/36-debug-transport-limits.md) |
| Std | `ScreenWidth`/`ScreenHeight` can report live terminal dimensions without resizing retained state | `console/operations/window.rs` | [37](tasks/37-console-screen-dimensions.md) |

## Confirmed P3 defect

| Area | Finding | Evidence | Task |
|---|---|---|---|
| CLI | Debug `--report jsonl` is accepted and discarded although JSONL is already the protocol output | `cli_input/options.rs`; debug help/examples | [31](tasks/31-debug-report-option.md) |

## Decision-required findings

These are plausible problems, but the current specification does not select the proposed behavior.
They are not implementation-ready bugfixes.

| Area | Question | Task |
|---|---|---|
| Sema | Are distinct named records nominally incompatible or structurally compatible? | [14](tasks/14-sema-named-records.md) |
| Sema | Must a public declaration that mentions a private type be rejected, or is an opaque public use allowed? | [18](tasks/18-sema-export-private-type.md) |
| VM | Does `Sleep` inside a synchronous callback suspend the whole spawned task or block its current worker? | [21](tasks/21-vm-callback-sleep.md) |
| Project | Is a physical source shared by consumer and dependency rejected, consumer-owned, or library-owned? | [23](tasks/23-project-origin.md) |
| CLI | Should test timeout include worker startup, and should tests have a default timeout? | [26](tasks/26-cli-test-timeout.md) |

## Coverage gap

`ensure_library_dependency` already rejects non-library dependencies, but path and workspace forms
lack direct regression coverage: [task 35](tasks/35-project-dependency-kind-tests.md).
