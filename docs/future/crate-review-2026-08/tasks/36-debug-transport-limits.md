# Task 36 — Bound DAP headers and JSONL request lines

Status: complete
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Debugger transports reject oversized input before allocating an unbounded `String`.

## Verified cause

- DAP bodies are capped at 16 MiB, but `dap/framing.rs` uses unbounded `read_line` for each header
  and accepts an unbounded number of headers.
- `jsonl/transport.rs` uses unbounded `read_line` for a complete request.

## Required implementation

- Add a shared transport-limit constants module only if both protocols genuinely share a value;
  keep protocol-specific parsing in their own files.
- DAP: bound individual header bytes, total header bytes/count, and continue enforcing the 16 MiB
  body cap. Reject before growing beyond the limit.
- JSONL: bound one complete request line at the existing maximum debugger message scale (16 MiB
  unless a smaller already-documented request limit is found), including the no-newline EOF case.
- Use `BufRead::fill_buf`/bounded reads or equivalent. Calling `read_line` and checking length after
  allocation does not satisfy the goal.
- Return deterministic invalid-data/protocol errors and terminate the affected transport cleanly.

Document exact public limits in debugger JSONL/DAP pages.

## Tests

- Oversized DAP header line and excessive header count fail before body allocation.
- Maximum valid DAP body still parses.
- Oversized JSONL line with and without trailing newline fails.
- A line exactly at the allowed boundary parses.

## Verify

```text
cargo test -p fpas-debug
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- No debugger input line/header has an unbounded growth path.
- Boundary tests cover accepted and rejected sizes.
- Docs name the enforced limits.

## Progress

- Base commit: 74b16b7b
- Current step: verify bounded DAP headers/bodies and JSONL request lines
- Files changed: debugger transport input/framing, DAP/JSONL tests, debugger protocol docs
- Verification: full workspace definition of done passed on 2026-08-19
- Blockers: none
