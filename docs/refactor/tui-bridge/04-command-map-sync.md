# 04 — Keep reserved `CM_*` list aligned with upstream

**Status:** [x] Initial 2.0 sync done · [x] Ongoing process documented · [x] Automate check (Rust test)

**Priority:** Ongoing — repeat on every turbo-vision version bump

**Depends on:** [done/01-turbo-vision-2-upgrade.md](done/01-turbo-vision-2-upgrade.md)

## Problem

FPAS offsets application-defined command ids that collide with upstream `CM_*` values (`command_map.rs`). Turbo Vision 2.0 **renumbered** standard commands to Borland values. A stale reserved list causes silent mis-routing or broken `OnCommand` round-trips.

## Current state (after 2.0 upgrade)

- `fpas-std/src/tui/command_ids.rs` — `Command.Quit=1`, `Close=4`, `Accept=10`, `Cancel=11`
- `command_map.rs` — reserved list from upstream `v2.0.0` `core/command.rs`; standard four pass through without offset
- Docs — [docs/pascal/std/tui/app/types.md](../../pascal/std/tui/app/types.md), [vm-bridge.md](../../pascal/std/tui/app/vm-bridge.md)

## Target

Repeatable checklist on every turbo-vision tag bump (crates.io or git).

## Tasks

- [x] Pin `turbo-vision` 2.0.0 (git tag until crates.io publishes 2.x)
- [x] Update `COMMAND_*` constants to Borland values
- [x] Refresh `TURBO_VISION_RESERVED_COMMANDS` for 2.0
- [x] Update Pascal docs for `Command.*` values
- [x] **Process** — Bump checklist in [00-context.md](00-context.md) and below
- [x] **Tests** — `reserved_list_matches_upstream_cm_constants` in `command_map.rs` diffs against upstream `CM_*` constants
- [x] **Automate check** — `cargo test -p fpas-vm reserved_list_matches_upstream` on every turbo-vision bump
- [x] **IDE** — `CmdHelpAbout = 100` (`CM_ABOUT`); About menu uses `CM_ABOUT`, close button `CM_OK` → `Command.Accept` ([done/03-about-message-box.md](done/03-about-message-box.md))

## Bump checklist (manual)

1. Bump `turbo-vision` in `Cargo.toml` / `Cargo.lock`.
2. Run `cargo test -p fpas-vm reserved_list_matches_upstream` — on failure, update `TURBO_VISION_RESERVED_COMMANDS` in `command_map.rs` from upstream [`src/core/command.rs`](https://github.com/aovestdipaperino/turbo-vision-4-rust/blob/v2.0.0/src/core/command.rs) (all non-zero `CM_*` except `CM_CONTINUE`).
3. Confirm `fpas_standard_command` pass-through set still matches Pascal `Command.*` constants.
4. Run:

   ```text
   cargo test -p fpas-vm command_map
   cargo run -q -p fpas-cli -- test tests/tui/controls/tui_turbo_vision_reserved_command_test.fpas
   ```

5. Update version mentions in docs (`1.3.1` → current).

## Files

```text
crates/fpas-std/src/tui/command_ids.rs
crates/fpas-vm/src/vm/execute/io/tui/command_map.rs
docs/pascal/std/tui/app/types.md
docs/pascal/std/tui/app/vm-bridge.md
Cargo.toml / Cargo.lock
```

## Notes

- Application code should use `Command.*` and named `const` ids, not raw Borland numbers (upstream 2.0 release notes).
- Custom ids in gaps (e.g. 15, 99, 200+) need no offset unless added to upstream `command.rs`.
