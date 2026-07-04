# fpas-std Review

## Summary

The crate passed package Clippy. The main concern is structural: several files now combine enough responsibilities to cross the repository size threshold.

## Findings

### Resolved: `Std.Str` implementation was a single oversized intrinsic dispatcher

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/str.rs` was 520 lines and contained the `Std.Str.*` intrinsic dispatcher starting at `crates/fpas-std/src/str.rs:20`. `Std.Str.Format` template expansion now lives in `crates/fpas-std/src/str/format.rs`, leaving `str.rs` (370 lines) and `str/format.rs` (156 lines).

Impact: String behavior spans search, edit, split/join, formatting, trimming, padding, and conversion. Keeping all of that in one dispatcher raises merge risk and makes behavior-specific review harder.

Follow-up: if `Std.Str` grows again, continue splitting `str.rs` into docs-shaped modules such as `case_trim`, `search`, `edit`, `split_join`, and `format_chars`.

### Resolved: Native graph backend file was large and mixed platform loop concerns

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/graph/backend/native.rs` had 636 lines. Native input mapping now lives in `graph/backend/native/input.rs`, and Softbuffer redraw/surface sizing now lives in `graph/backend/native/surface.rs`, leaving `native.rs` (362 lines), `native/input.rs` (214 lines), and `native/surface.rs` (78 lines).

Impact: Window creation, event pumping, surface resize, frame upload, and error conversion live together. Native backend changes are likely to touch unrelated concerns.

Resolution: split by ownership while keeping public graph session code unchanged.

### Medium: Additional `fpas-std` files still exceed the structure threshold

Evidence: after resolving the original findings, a recursive scan still reports `crates/fpas-std/src/std_units/symbols/std_symbols.rs` (585 lines), `crates/fpas-std/src/tui/tests.rs` (414 lines), and `crates/fpas-std/src/std_units/symbols/groups.rs` (407 lines). Resolved for `crates/fpas-std/src/graph/session.rs`: hosted redraw/event queue methods moved to `graph/session/events.rs`, leaving `session.rs` (325 lines) and `session/events.rs` (104 lines).

Impact: the remaining files are close to or above the repository's preferred 400-line split point. `graph/session.rs` is production runtime state and should be handled before generated or test-only structure cleanup.

Next step: inspect the `std_units/symbols` files to decide whether they are generated/registry data that should be split by symbol group, then address the test-only `tui/tests.rs`.

## Verification

- `cargo clippy -p fpas-std --all-targets -- -D warnings` passed.
