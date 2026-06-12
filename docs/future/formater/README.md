# Source formatter (`fpas fmt`)

Planning and implementation for a canonical Functional Pascal source formatter (`fpas fmt`). CLI: [cli.md](cli.md).

## Status

**v1 and v2 complete; v3 Phase 0 complete.** Implementation **not started** (Phase 1 next). Plan: [implementation-v3.md](implementation-v3.md) — full trivia + editor integration. Emitter [`crates/fpas-fmt/`](../../../crates/fpas-fmt/); CLI [`fpas fmt`](../../../crates/fpas-cli/src/cli_fmt/) per [cli.md](cli.md). CI: [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml). Tests: [`round_trip.rs`](../../../crates/fpas-fmt/tests/round_trip.rs), [`fuzz_light.rs`](../../../crates/fpas-fmt/tests/fuzz_light.rs).

## Principles

- **AST pretty-printer with v3 trivia layer (planned).** v2: parse → emit canonical text; v3 adds trivia preservation for comments and user blank lines ([implementation-v3.md](implementation-v3.md)).
- **One `.fpas` file = one compilation unit.** Project/workspace loading is a CLI concern; the formatter crate formats a single parsed unit.
- **No backward-compatibility requirement.** Comments, keyword casing, and literal spelling (`$FF` vs `255`, `1_000` vs `1000`) are not preserved — output follows [style.md](style.md).
- **Explicit `begin` / `end`.** Where the language allows a single statement after `then`, `else`, `do`, or a `case` label, the formatter still emits a `begin` / `end` block ([style.md — Blocks](style.md#blocks-begin--end)).
- **Blank lines (v2).** After `program` / `unit` and after `uses`: exactly one blank line each; one blank line between record fields and methods; other user-placed blank lines not preserved until **v3** ([style.md — Comments (v3 planned)](style.md#comments-v3-planned)).
- **Parse errors block formatting.** Same diagnostics surface as `fpas check` / `fpas_parser::parse_compilation_unit`.
- **Deterministic output.** Same input AST always yields the same text (stable ordering, 2-space indent).
- **One official style.** [style.md](style.md) is normative; no `.fpasfmt.toml` or per-project overrides (v1 and v2).

## Documents

| File | Purpose |
|------|---------|
| [style.md](style.md) | **Normative** formatting rules and examples (edit here first) |
| [implementation.md](implementation.md) | Phased task list for v1 (`fpas-fmt`) — **complete** |
| [implementation-v2.md](implementation-v2.md) | Phased task list for v2 (CLI, wrapping, comments, CI) — **complete** |
| [implementation-v3.md](implementation-v3.md) | Phased task list for v3 (full trivia, editor integration) — **Phase 0 done; Phase 1 next** |
| [cli.md](cli.md) | CLI usage and exit codes |

## Related

- Language rules: [`docs/pascal/`](../../pascal/)
- Control flow (optional vs required blocks): [`docs/pascal/03-control-flow.md`](../../pascal/03-control-flow.md)
- Semicolons: [`docs/pascal/02-basics.md`](../../pascal/02-basics.md)
- Projects: [`docs/pascal/10-projects.md`](../../pascal/10-projects.md)
- Parser AST: [`crates/fpas-parser/src/ast/`](../../../crates/fpas-parser/src/ast/)
- Existing CLI patterns: [`crates/fpas-cli/src/cli_input.rs`](../../../crates/fpas-cli/src/cli_input.rs), [`cli_check.rs`](../../../crates/fpas-cli/src/cli_check.rs)
