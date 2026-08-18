# Task 03 — FromChar, Pad*, IntToHex: spec + allocation cap

Status: open
Severity: P1 (FromChar) / P2 (Pad / IntToHex)
Difficulty: easy
Language gate: no
Depends on: none

## Goal

1. `FromChar(C, Count)` with `Count ≤ 0` returns `''` (same as `RepeatStr`).
2. Positive `FromChar` / `PadLeft` / `PadRight` / `PadCenter` / `IntToHex` widths that would allocate unbounded memory return a runtime error instead of panicking.

## Spec

- [`docs/pascal/std/text/str/format-chars.md`](../../../pascal/std/text/str/format-chars.md): `Count ≤ 0` yields `''`.
- [`docs/pascal/std/text/str/edit.md`](../../../pascal/std/text/str/edit.md): `RepeatStr` caps positive count at **1_000_000**.

Reuse `checked_collection_len` / `MAX_COLLECTION_LEN` in `crates/fpas-std/src/limits.rs`. If Pad/IntToHex were previously unbounded with no documented cap, add one sentence to the matching std page (same 1_000_000 cap as `RepeatStr`) — that is documenting the implemented limit, not a new language feature.

## Bug

`crates/fpas-std/src/str.rs` `FromChar`: `Count < 0` is a domain error; large positive `Count` does `repeat_n(..., n as usize)` with no cap.

`checked_pad_width` (same file) only rejects `Width < 0`. `IntToHex` in `crates/fpas-std/src/conv.rs` has the same Digits problem.

## Fix

- `FromChar`: if `n <= 0` return empty string; else `let len = checked_collection_len(n, location, "Std.Str.FromChar")?;` then repeat.
- Pad / IntToHex: run the width/digits through the same cap (or `checked_pad_width` calling `checked_collection_len`). Negative pad width stays an error if that is current behavior; do not change FromChar negatives to errors.

Do not raise the cap. Do not allocate and then truncate.

## Tests

- Rust or FPAS: `FromChar('x', 0)` and `FromChar('x', -1)` both `''`.
- `FromChar('x', 1_000_001)` runtime error (do not use `i64::MAX` if that still overflows `as usize` before the check — check first, then convert).
- PadLeft or IntToHex with width/digits `1_000_001` runtime error.
- Existing `from_char_zero_count_test.fpas` and small pad/hex tests must still pass.

## Verify

```text
cargo test -p fpas-std
fpas test tests/stdlib/
cargo fmt
```

Target the specific `*_test.fpas` files if the full folder is slow.

## Done when

- Negatives/zero for FromChar match the spec.
- Huge counts/widths error via `checked_collection_len`.
- Std docs mention the cap for any API you newly capped.
