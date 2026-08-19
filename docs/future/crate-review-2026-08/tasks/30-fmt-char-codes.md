# Task 30 — Emit Pascal character codes for control characters

Status: complete

## Progress

- Implementation commit: 74b16b7b
- Current step: complete; remove this task after the completion cleanup is committed
- Verification: cargo fmt --all -- --check, cargo build --workspace, and
  cargo test --workspace --no-fail-fast passed on 2026-08-19
- Docs: current user-facing documentation was included or confirmed by the implementation slice
- Blockers: none
Severity: P2
Difficulty: easy
Language gate: no
Depends on: none

## Goal

Formatting a parsed string containing control characters emits Pascal character-code segments such
as `#0`, `#9`, and `#13#10`; formatted source contains no raw NUL, CR, LF, or tab inside quotes.

## Contract

[`Formatter style`](../../../pascal/tools/fmt-style.md) and the string-literal grammar accept quoted
segments concatenated with `#` character codes.

## Verified cause

`fpas-fmt/src/emit/expr/literal.rs::format_string` writes every scalar except apostrophe literally
inside one quoted segment.

## Fix

Split output into quoted printable segments and decimal `#<codepoint>` segments. Escape apostrophes
as doubled quotes. Preserve printable Unicode literally; **do not** classify all non-ASCII text as
unprintable. Handle empty strings and adjacent control characters without producing syntactically
invalid empty/unterminated segments.

## Tests

- Round-trip `#0`, `#9`, `#13#10`, controls between printable text, apostrophes, and printable
  Unicode.
- Assert formatted bytes contain no raw control byte except file newlines.
- Formatting a second time is byte-identical.

## Verify

```text
cargo test -p fpas-fmt
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Control values round-trip through valid FPAS source.
- Printable Unicode remains readable.
- Formatter idempotence holds.
