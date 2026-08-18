# Task 02 — Slice / Substring / Delete must not overflow

Status: open
Severity: P1
Difficulty: easy
Language gate: no
Depends on: none

## Goal

Out-of-range `Slice`, `Std.Str.Substring`, and `Std.Str.Delete` are runtime errors even when `start + len` overflows `i64`. They must not panic.

## Spec

- [`docs/pascal/std/collections/array/combine.md`](../../../pascal/std/collections/array/combine.md) (Slice)
- Substring / Delete pages under `docs/pascal/std/text/str/`

An out-of-range range is a **runtime error**.

## Bug

`crates/fpas-std/src/array.rs` (`Slice` around the `start + len` check) and `crates/fpas-std/src/str.rs` (`Substring`, `Delete`): the check uses `i64` addition. `Slice(A, 1, 9223372036854775807)` overflows (debug panic / release wrap) instead of `StdError`.

Existing FPAS tests only use modest out-of-range values:

- `tests/stdlib/array/slice_out_of_bounds_is_runtime_error_runtime_error.fpas`
- `tests/stdlib/.../substring_out_of_bounds_is_runtime_error_runtime_error.fpas` (find the real path)
- `tests/stdlib/.../delete_out_of_bounds_runtime_error.fpas` (find the real path)

## Fix

Use `checked_add` (or compare without adding: `len > n - start` after validating `start`/`len`/`n` are in range). Reject negative `start`/`len` as today. Index with `as usize` only after the range is proven in-bounds.

Do not change the error code family already used for these APIs.

## Tests

Prefer small Rust unit tests in `fpas-std` that call the intrinsic with `i64::MAX` (faster and does not need a huge array). Also add or extend an FPAS runtime-error test with `start = 0` or `1` and `len = 9223372036854775807` on a tiny array/string.

## Verify

```text
cargo test -p fpas-std
fpas test tests/stdlib/array/
fpas test tests/stdlib/
cargo fmt
```

If the full stdlib suite is slow, at least run the Slice/Substring/Delete test files you touched plus `cargo test -p fpas-std`.

## Done when

- `start + len` overflow is a runtime error, not a panic.
- Existing modest-OOB tests still pass.
- Docs unchanged unless a page claimed wrapping (it does not).
