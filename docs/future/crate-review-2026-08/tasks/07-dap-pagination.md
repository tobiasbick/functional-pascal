# Task 07 — DAP `levels` / `count` of 0 means “all”

Status: open
Severity: P1
Difficulty: easy
Language gate: no
Depends on: none

## Goal

Omitted or `0` `stackTrace.levels` and `variables.count` return the full page, not an empty list.

## Spec

Debug Adapter Protocol: `0` means unpaginated / all items. Clients send `0` expecting frames and variable children.

## Bug

`crates/fpas-debug/src/dap/server/dispatch.rs` forwards `0` into JSONL `count`. The VM uses `.take(count)`, so `0` yields nothing.

## Fix

When DAP `levels` / `count` is `None` or `0`, omit the cap or pass a documented “all” sentinel that the JSONL/VM side already uses when the field is missing. Do not treat `0` as `take(0)`. Positive counts stay page sizes.

Search for `take(count)` / `count` in the debug VM inspection path and fix the interpretation at the DAP boundary (preferred) or at `.take` (if JSONL should match DAP too). If JSONL currently documents `0` as empty, keep JSONL as-is and only fix DAP — then say so in the test comments. Prefer one consistent meaning: `0` = all in both DAP and JSONL unless JSONL tests already assert otherwise (do not break those; then DAP-only).

## Tests

DAP inspection test: `levels: 0` / `count: 0` returns the same frames/variables as omitting the field (or as a large count). Existing tests that omit `count` or send `20` must still pass.

## Verify

```text
cargo test -p fpas-debug
cargo fmt
```

## Done when

- DAP `0` returns all items.
- Docs: one sentence in `docs/pascal/tools/debugger.md` or DAP notes if pagination is documented; else unchanged.
