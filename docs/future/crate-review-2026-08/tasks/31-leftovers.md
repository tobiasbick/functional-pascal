# Task 31 — Small leftovers (P3 and cap DAP/JSONL lines)

Status: open
Severity: P3 / P2
Difficulty: easy
Language gate: no
Depends on: none

Do these in one change set. None of them need design.

## 31a — `--report jsonl` on `fpas debug`

`crates/fpas-cli/src/cli_input/options.rs`: the flag is accepted as `jsonl`, never stored. Help and debugger docs show it on `--commands` examples.

**Fix:** either wire it (only if JSONL output already exists for debug reports) **or** remove it from clap/help/docs so agents are not lied to. Prefer **remove** if it is a no-op.

## 31b — Non-library `[dependencies]`

`crates/fpas-project/src/dependencies.rs` `ensure_library_dependency` already implements the spec. Add a test that a `kind = "program"` path in `[dependencies]` errors. Same for `workspace = ["<program member>"]` if that is one extra assertion.

## 31c — Cap DAP/JSONL header and line size

`crates/fpas-debug/src/dap/framing.rs` and `crates/fpas-debug/src/jsonl/transport.rs`: `read_line` is unbounded. Bodies are already capped at 16 MiB. Cap a single header line / JSONL line (e.g. 1 MiB) and a header count; return a protocol error. Test oversized header if easy.

## 31d — ScreenWidth vs back buffer (optional skip)

`crates/fpas-std/src/console/operations/io.rs`: with a writer attached, `ScreenWidth`/`Height` query live `crossterm` size without `sync_terminal_size`. **Skip** if you cannot mock terminal size without a TTY. If you can call `sync_terminal_size` from those getters, do that and add a unit test with the existing console test double.

## Verify

```text
cargo test -p fpas-cli
cargo test -p fpas-project
cargo test -p fpas-debug
cargo fmt
```

## Done when

- Help matches behavior for `--report`.
- Non-library dependency has a test.
- Giant DAP/JSONL lines error instead of unbounded alloc.
- 31d done or explicitly skipped in the closing note.
