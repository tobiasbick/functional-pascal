---
name: turbo-vision-4-rust
description: Use when extending or debugging Functional Pascal `Std.Tui` over the Rust `turbo-vision` crate from `aovestdipaperino/turbo-vision-4-rust`, especially tasks touching `docs/pascal/std/tui`, TUI sema/compiler/bytecode/VM/runtime code, TUI tests, examples, or `apps/ide`. Also use when checking upstream Turbo Vision API, dependency, license, or crossterm compatibility, or closing the three remaining `bridged_*` read-back adapters.
---

# Turbo Vision 4 Rust

Guide for extending and maintaining the FPAS-native `Std.Tui` facade over upstream `turbo-vision`. Treat this skill as project-local integration procedure, not as upstream API documentation.

## Required First Reads

Before edits, read these files in order:

1. `docs/pascal/std/tui/README.md`
2. `docs/pascal/std/tui/app/vm-bridge.md` — canonical bridge file map
3. `docs/pascal/std/tui/terminal-checklist.md` — local verification commands after bridge changes
4. `.agents/skills/fpas-change-checklist/SKILL.md` when implementing or modifying behavior, public API, docs under `docs/pascal/`, tests, compiler, VM, runtime, or stdlib code.

If the task is about examples of how to apply the skill, read `references/api_reference.md`.

For upstream `message_box` / IDE About work, see [message-box.md](../../../docs/pascal/std/tui/app/message-box.md) and [modals.md](../../../docs/pascal/std/tui/app/modals.md).

## Upstream Verification Rule

Do not rely on model memory for `turbo-vision` API details. It is a young crate and may change.

When adding or changing integration code, verify the current upstream state from primary sources:

- GitHub repository: `https://github.com/aovestdipaperino/turbo-vision-4-rust`
- crates.io crate: `turbo-vision`
- local `Cargo.lock` / `cargo tree` after dependency changes

Minimum facts to refresh when relevant:

- latest version and yanked status
- crate name `turbo-vision` and Rust library name `turbo_vision`
- dependency versions, especially `crossterm`
- available modules and public constructors used by the integration
- license text and copyright notice if vendoring or copying code

Use official upstream files over summaries. Prefer `Cargo.toml`, `src/lib.rs`, module source files, examples, and crates.io metadata.

## Architecture Decision

Implement an FPAS-native API over Turbo Vision concepts. Do not mirror the Rust API one-to-one.

Use:

- host-owned handles for live UI objects;
- FPAS records for values such as `Rect`, `Point`, `Size`, and events;
- command constants or a simple command type;
- explicit FPAS callback registration for commands/events;
- Turbo Vision's application, desktop, window, dialog, menu, status line, control, event, and modal concepts.

Avoid:

- exposing Rust traits, `Box<dyn View>`, builders, ownership details, or Rust module layout;
- preserving the old retained-view API for compatibility;
- creating broad adapters for `Application.Host*` calls;
- documenting planned behavior in `docs/pascal/` before it exists.

## Current Status

The Turbo Vision rewrite is **functionally complete**. The bridge under `crates/fpas-vm/src/vm/execute/io/tui/bridge/` is live and headless-testable.

Remaining work is narrow:

- three `bridged_*` adapters (`CheckBox`, `RadioButton`, `Outline`) until upstream exposes live read-back — see [tui-bridged-readback.md](../../../docs/future/tui-bridged-readback.md);
- periodic upstream checks before declaring Stream A closed ([AGENTS.md](../../../AGENTS.md#upstream-watch--turbo-vision-4-rust-read-back-stream-a)).

For new `Std.Tui` work:

1. Read the current spec under `docs/pascal/std/tui/`.
2. Follow the end-to-end recipe in `docs/pascal/std/tui/app/vm-bridge.md`.
3. Extend tests that assert user-visible behavior, not deleted retained-engine internals.

## File and API Discipline

Follow project `AGENTS.md` structure rules:

- keep one concern per file;
- split large files before adding more mixed logic;
- prefer subdirectories over crowded modules;
- remove dead code introduced or exposed by the rewrite.

The current bridge is Turbo Vision only: `tui/mod.rs` dispatches to `bridge/`; no legacy root bridge modules remain. Use [vm-bridge.md](../../../docs/pascal/std/tui/app/vm-bridge.md) for the authoritative file map — do not duplicate it here. Do not reintroduce `reconcile.rs`, `ExecDialog`, or command-offset routing.

Key bridge areas (details in vm-bridge.md):

| Concern | Location |
| --- | --- |
| Intrinsic dispatch | `mod.rs`, `bridge/application_intrinsics.rs`, `bridge/intrinsics.rs` |
| Session lifecycle | `bridge/lifecycle.rs`, `bridge/session_app.rs` |
| View registry | `bridge/session.rs`, `bridge/registry.rs`, `bridge/views/` |
| Run loop / input | `bridge/run.rs`, `bridge/input_events.rs` |
| Commands / chrome | `bridge/events.rs`, `bridge/chrome*.rs` |
| Modals | `bridge/modals.rs`, `bridge/message_box.rs`, `bridge/file_dialog.rs` |
| Headless | `bridge/headless_draw.rs`, `bridge/testing.rs` |
| Upstream read-back adapters | `bridge/bridged_check_box.rs`, `bridge/bridged_radio_button.rs`, `bridge/bridged_outline.rs` |

Do not replace the three adapters with snapshot/reconcile code. Closure steps: [tui-bridged-readback.md](../../../docs/future/tui-bridged-readback.md) and [AI_CONTRIBUTING.md](../../../AI_CONTRIBUTING.md#good-entry-points).

`Application.MessageBox` is documented in [message-box.md](../../../docs/pascal/std/tui/app/message-box.md).

## Documentation Rules

- Update `docs/pascal/std/tui/` for implemented behavior only.
- Keep speculative plans in `docs/future/`.
- Update Rust `///` docs that link to `docs/pascal/std/tui/` paths.

## Testing Rules

Prefer:

- Rust tests for bridge invariants, invalid handles, callback re-entry, cleanup, and diagnostics (`bridge/testing.rs`, `cargo test -p fpas-vm bridge::`).
- FPAS tests for public behavior under `tests/tui/` — themed subdirs: `views/`, `events/`, `smoke/`, `modals/`.
- Headless tests over manual terminal checks; run [terminal-checklist.md](../../../docs/pascal/std/tui/terminal-checklist.md) after bridge edits.

Do not preserve tests that only validate old retained-view internals such as `QuerySceneGraph`, retained clip state, `HostProcessNext` integer tags, or old frame-root query records.

## License Rule

MIT is compatible with this workspace's BSD-3-Clause license. When using `turbo-vision` as a dependency, keep dependency metadata and lockfile accurate. When copying or vendoring upstream code, preserve MIT license and copyright notices.
