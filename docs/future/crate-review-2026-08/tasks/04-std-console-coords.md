# Task 04 — Window and GotoXY must error on bad coordinates

Status: open
Severity: P1
Difficulty: easy
Language gate: no
Depends on: none

## Goal

`Window` and `GotoXY` raise a runtime error when coordinates are outside the documented range. They must not return `Ok(())` and leave state unchanged.

## Spec

[`docs/pascal/std/console/screen.md`](../../../pascal/std/console/screen.md)

- `GotoXY(X, Y)`: runtime error if `X` / `Y` are outside the **active window** (1-based, window-relative).
- `Window(X1, Y1, X2, Y2)`: runtime error if the rectangle is outside the current screen **or** `X1 > X2` / `Y1 > Y2`.

Inverted rectangles already error. Out-of-screen / out-of-window currently hit `check_coord` and return success.

## Bug

- `crates/fpas-std/src/console/operations/window.rs` — coordinates that fail `check_coord` return `Ok(())`.
- `crates/fpas-std/src/console/operations/cursor.rs` — `GotoXY` same.

`check_coord` is appropriate for resize clipping on some reads; it is the wrong policy for these two mutating APIs.

## Fix

For `Window` and `GotoXY` only: if coordinates are invalid, return `std_runtime_error` (reuse the console/runtime code already used for inverted windows if one exists; otherwise the same family as other coordinate errors). Do not change `WhereX` / cell queries.

## Tests

Add Rust tests next to existing console operation tests (search `Window(` / `GotoXY` under `crates/fpas-std`). Cover:

- `GotoXY(0, 1)` and a coordinate past the window size → error.
- `Window` with `X2 > ScreenWidth` (or X1 = 0) → error.
- Valid `Window` then `GotoXY(1, 1)` still succeeds.

Optional FPAS test under `tests/stdlib/` / `tests/console/` if that is how other CRT tests run (headless). Prefer crate tests if graph/console host is required.

## Verify

```text
cargo test -p fpas-std
cargo fmt
```

Plus any FPAS file you added: `fpas test <path>`.

## Done when

- Invalid Window/GotoXY are errors.
- Docs unchanged (already specify the errors).
