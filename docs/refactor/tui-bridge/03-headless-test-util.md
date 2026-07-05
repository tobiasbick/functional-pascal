# 03 — Headless tests via turbo-vision `test-util`

**Status:** [ ] Not started · [ ] In progress · [ ] Done

**Priority:** Medium–high (large refactor, high long-term payoff)

**Depends on:** [done/02-single-tv-session.md](done/02-single-tv-session.md) recommended so headless and live share one draw/event path.

**Blocks:** Retiring most of `headless_paint.rs`; simpler `AssertScreenCell` semantics aligned with TV.

## Problem

Headless `Application.OpenForTest` + `Run` uses a **parallel implementation**:

| Live | Headless today |
| --- | --- |
| TV `desktop.draw` + crossterm | `headless_paint.rs` writes ASCII-ish cells to CRT buffer |
| TV `handle_event` | Command queue + `TestClickButton` / `TestDispatchMenuCommand` |
| Bridged views for mouse | Coordinate hit-test in `test_mouse.rs` |

~200+ LOC of paint logic duplicates upstream layout rules (menu titles, dialog frames, button labels). Drift causes false positives/negatives in `AssertScreenCell` tests.

## Target

Enable turbo-vision **`test-util`** feature on the workspace dependency:

```toml
turbo-vision = { git = "…", tag = "v2.0.0", features = ["test-util"] }
```

Use `turbo_vision::test_util::MockTerminal`:

- Build the same TV view tree as interactive mode (from FPAS snapshots).
- Call upstream `draw` into `MockTerminal`.
- Copy mock buffer to FPAS CRT for `Std.Test` assertions (or assert on mock directly in Rust tests).

Reduce or remove `headless_paint.rs` when parity is proven.

## Tasks

- [ ] **Dependency** — Add `test-util` feature to workspace `turbo-vision` dependency; confirm license/build impact (MIT upstream).
- [ ] **Prototype** — One FPAS test (`tui_turbo_vision_chrome_paint_test.fpas` or Rust integration test) drawing menu bar via MockTerminal.
- [ ] **Session wiring** — Headless `OpenForTest` attaches MockTerminal to session `Application` instead of skipping TV init entirely (coordinate with 01).
- [ ] **Port paint** — Replace `turbo_vision_paint_headless_desktop` call sites with TV draw → buffer export.
- [ ] **Input** — Route `TestClickButton` / `TestClickMouse` through TV event injection (`MockTerminal::push_event`) where possible; drop duplicate hit-test where upstream handles it.
- [ ] **Delete** — Remove dead code from `headless_paint.rs` (file or most of it).
- [ ] **Regression** — Full `tests/tui/controls/` suite; fix golden/cell expectations if TV draw differs slightly from old painter.
- [ ] **Docs** — [docs/pascal/std/tui/app/testing.md](../../pascal/std/tui/app/testing.md) if headless rules change.
- [ ] **Context** — Update [00-context.md](00-context.md).

## Files (expected touch)

```text
Cargo.toml
crates/fpas-vm/Cargo.toml
crates/fpas-vm/src/vm/execute/io/tui/headless_paint.rs   (shrink or delete)
crates/fpas-vm/src/vm/execute/io/tui/tv_run.rs
crates/fpas-vm/src/vm/execute/io/tui/interactive_loop.rs
crates/fpas-vm/src/vm/execute/io/tui/test_mouse.rs
tests/tui/controls/*
```

## Verification

```text
cargo test --workspace
cargo run -q -p fpas-cli -- test tests/tui/controls/
```

## Risks

- MockTerminal API may not expose every crossterm behavior (colors, wide chars) — document gaps instead of reimplementing.
- Feature flag only for fpas-vm dev/test if binary size matters (unlikely for compiler VM).

## Notes

- Keep FPAS-level `TestSetDialogResult` until modals use shared session draw path.
- Upstream doc: `turbo-vision` `test_util.rs` on tag `v2.0.0`.
