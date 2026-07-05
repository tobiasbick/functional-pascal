# Done — Upgrade to turbo-vision 2.0 + Borland command ids

**Status:** [x] Done (2026-07)

## Summary

Pinned upstream [turbo-vision-4-rust v2.0.0](https://github.com/aovestdipaperino/turbo-vision-4-rust/releases/tag/v2.0.0) (git dependency until crates.io publishes 2.x). Aligned FPAS `Command.*` constants with Borland `CM_*` values and refreshed the VM reserved-command map.

## Completed tasks

- [x] Workspace dependency: `turbo-vision = { git = "…", tag = "v2.0.0" }`
- [x] `fpas-std/src/tui/command_ids.rs` — Quit=1, Close=4, Accept=10, Cancel=11
- [x] `command_map.rs` — TV 2.0 reserved list; standard four pass through without offset
- [x] IDE `CmdFileExit` → 1
- [x] Docs: `docs/pascal/std/tui/`, root README third-party table
- [x] Verify: `cargo test --workspace`, `fpas test tests/tui/controls/`, IDE tests, regression suite

## Breaking change for FPAS authors

| Constant | Old | New |
| --- | --- | --- |
| `Command.Quit` | 4 | 1 |
| `Command.Close` | 3 | 4 |
| `Command.Accept` | 1 | 10 |
| `Command.Cancel` | 2 | 11 |

Code using symbolic `Command.*` needs no change. Numeric literals must be updated.

## Follow-ups (not part of this item)

See open items in [../README.md](../README.md): message_box, headless test-util, etc. Single live session per FPAS app: [02-single-tv-session.md](02-single-tv-session.md) (done).

## Commit reference

Search git history for “turbo-vision 2.0” / “Borland command” if commit hash needed.
