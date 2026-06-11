# Source formatter (`fpas fmt`)

Planning and implementation for a canonical Functional Pascal source formatter (`fpas fmt`). CLI: [cli.md](cli.md).

## Status

**v1 complete.** Emitter [`crates/fpas-fmt/`](../../../crates/fpas-fmt/); CLI [`fpas fmt`](../../../crates/fpas-cli/src/cli_fmt.rs) per [cli.md](cli.md). Golden examples: [style.md — Formatted output](style.md#formatted-output-fpas-fmt).

## Principles

- **AST pretty-printer**, not a trivia-preserving token rewriter. Parse → emit canonical text.
- **One `.fpas` file = one compilation unit.** Project/workspace loading is a CLI concern; the formatter crate formats a single parsed unit.
- **No backward-compatibility requirement.** Comments, keyword casing, and literal spelling (`$FF` vs `255`, `1_000` vs `1000`) are not preserved — output follows [style.md](style.md).
- **Explicit `begin` / `end`.** Where the language allows a single statement after `then`, `else`, `do`, or a `case` label, the formatter still emits a `begin` / `end` block ([style.md — Blocks](style.md#blocks-begin--end)).
- **Fixed blank lines.** After `program` / `unit` and after `uses`: exactly one blank line each; one blank line between record fields and methods; no other user-placed blank lines preserved ([style.md — Blank lines](style.md#blank-lines)).
- **Parse errors block formatting.** Same diagnostics surface as `fpas check` / `fpas_parser::parse_compilation_unit`.
- **Deterministic output.** Same input AST always yields the same text (stable ordering, 2-space indent).

## Documents

| File | Purpose |
|------|---------|
| [style.md](style.md) | **Normative** formatting rules and examples (edit here first) |
| [implementation.md](implementation.md) | Phased task list for `fpas-fmt` |
| [cli.md](cli.md) | CLI input, discovery, globs — **discussion deferred** |

## Related

- Language rules: [`docs/pascal/`](../../pascal/)
- Control flow (optional vs required blocks): [`docs/pascal/03-control-flow.md`](../../pascal/03-control-flow.md)
- Semicolons: [`docs/pascal/02-basics.md`](../../pascal/02-basics.md)
- Projects: [`docs/pascal/10-projects.md`](../../pascal/10-projects.md)
- Parser AST: [`crates/fpas-parser/src/ast/`](../../../crates/fpas-parser/src/ast/)
- Existing CLI patterns: [`crates/fpas-cli/src/cli_input.rs`](../../../crates/fpas-cli/src/cli_input.rs), [`cli_check.rs`](../../../crates/fpas-cli/src/cli_check.rs)
