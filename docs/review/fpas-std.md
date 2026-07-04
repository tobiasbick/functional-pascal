# fpas-std Review

## Summary

The crate passed package Clippy. Structural remediation for runtime modules, the `Std.*` symbol registry, and TUI session tests is complete. No `fpas-std` Rust source file currently exceeds the repository's 400-line split threshold.

## Findings

### Resolved: `Std.Str` implementation was a single oversized intrinsic dispatcher

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/str.rs` was 520 lines and contained the `Std.Str.*` intrinsic dispatcher starting at `crates/fpas-std/src/str.rs:20`. `Std.Str.Format` template expansion now lives in `crates/fpas-std/src/str/format.rs`, leaving `str.rs` (370 lines) and `str/format.rs` (156 lines).

Impact: String behavior spans search, edit, split/join, formatting, trimming, padding, and conversion. Keeping all of that in one dispatcher raises merge risk and makes behavior-specific review harder.

Follow-up: if `Std.Str` grows again, continue splitting `str.rs` into docs-shaped modules such as `case_trim`, `search`, `edit`, `split_join`, and `format_chars`.

### Resolved: Native graph backend file was large and mixed platform loop concerns

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/graph/backend/native.rs` had 636 lines. Native input mapping now lives in `graph/backend/native/input.rs`, and Softbuffer redraw/surface sizing now lives in `graph/backend/native/surface.rs`, leaving `native.rs` (362 lines), `native/input.rs` (214 lines), and `native/surface.rs` (78 lines).

Impact: Window creation, event pumping, surface resize, frame upload, and error conversion lived together. Native backend changes were likely to touch unrelated concerns.

Resolution: split by ownership while keeping public graph session code unchanged.

### Resolved: `GraphSession` mixed drawing state with hosted event queue APIs

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/graph/session.rs` had hosted redraw and event-queue methods alongside drawing and upload APIs. Those methods now live in `graph/session/events.rs`, leaving `session.rs` (325 lines) and `session/events.rs` (104 lines).

Impact: Hosted-loop event handling and backbuffer drawing were coupled in one runtime state file.

Resolution: move hosted redraw/event queue methods into `graph/session/events.rs`.

### Resolved: `std_units/symbols` registry was split across two oversized files

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/std_units/symbols/std_symbols.rs` had 585 lines and `crates/fpas-std/src/std_units/symbols/groups.rs` had 407 lines. Symbol constants and per-unit `STD_*_SYMBOLS` arrays now live together under `crates/fpas-std/src/std_units/symbols/std_symbols/` in one file per `Std.*` unit (for example `console.rs` at 171 lines, `tui.rs` at 309 lines, `mod.rs` at 147 lines including shared macros). `groups.rs` was removed; `symbols/mod.rs` re-exports the symbol arrays.

Impact: Adding or reviewing symbols for one unit required scrolling a monolithic registry and a separate group list.

Resolution: colocate each unit's constants and symbol group; keep macro helpers in `std_symbols/mod.rs`.

### Resolved: `tui/tests.rs` mixed session lifecycle, redraw, and event tests

Evidence: resolved on 2026-07-04. `crates/fpas-std/src/tui/tests.rs` had 414 lines covering deferred/headless open, redraw damage, resize redraw, and console event mapping. Tests now live under `crates/fpas-std/src/tui/tests/` in `helpers.rs` (28 lines), `lifecycle.rs` (72 lines), `redraw.rs` (228 lines), and `events.rs` (98 lines).

Impact: Session lifecycle, damage tracking, and event translation tests were coupled in one file.

Resolution: split by test theme while sharing `helpers.rs` for console fixtures.

### Low: `console/screen/mod.rs` is close to the structure threshold

Evidence: `crates/fpas-std/src/console/screen/mod.rs` has 392 lines.

Impact: The file is still under the 400-line split point but may cross it with the next console screen feature.

Next step: split `console/screen/mod.rs` only if it grows past 400 lines during the next console change.

## Verification

- `cargo clippy -p fpas-std --all-targets -- -D warnings` passed after the `tui/tests` split.
- `cargo test -p fpas-std tui_session` passed (16 tests).
