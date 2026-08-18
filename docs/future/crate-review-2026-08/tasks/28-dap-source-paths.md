# Task 28 — DAP source matching: case-insensitive suffix + remapped paths

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Breakpoints bind on Windows when the client path casing differs from the stored project path. Out-of-tree library sources remapped to `sources/{index}/{filename}` still match when the IDE sends the real path.

## Bug

`crates/fpas-debug/src/dap/server/breakpoints.rs` `resolve_source_path`: exact match or case-sensitive `ends_with("/" + stored)`. VS Code may send `src/Main.fpas` vs stored `src/main.fpas`.

`crates/fpas-cli/src/cli_debug.rs` `portable_path` rewrites sources outside the project root to `sources/{index}/{filename}`. An IDE sending the real library path will not suffix-match that alias.

Tests use `"<memory>"` and a matching `C:/workspace/<memory>` suffix only.

## Fix

- Compare paths case-insensitively on Windows (or always case-insensitive for ASCII drive/file names — match how the rest of FPAS compares paths).
- When resolving DAP sources, also try the **original** path recorded before `portable_path`, if it is stored. If not stored, store it when building the debug source map (cli_debug.rs) and read it in the adapter.

Do not require the user to set breakpoints on the alias path only.

## Tests

- Same path with different casing binds.
- A remapped `sources/0/lib.fpas` binds when the client sends the original `../libs/lib.fpas` (use the same helpers portable_path tests use).

## Verify

```text
cargo test -p fpas-debug
cargo test -p fpas-cli
cargo fmt
```

## Done when

- Case mismatch binds on Windows-style paths.
- Real library path binds after remap.
- Docs: one sentence in debugger/editor-integration if source aliases are documented.
