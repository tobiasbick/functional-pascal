---
name: turbo-vision-4-rust
description: Use when extending or debugging Functional Pascal `Std.Tui` over the Rust `turbo-vision` crate from `aovestdipaperino/turbo-vision-4-rust`, especially tasks touching `docs/pascal/std/tui`, TUI sema/compiler/bytecode/VM/runtime code, TUI tests, examples, or `apps/ide`. Also use when checking upstream Turbo Vision API, dependency, license, or crossterm compatibility.
---

# Turbo Vision 4 Rust

Guide for extending and maintaining the FPAS-native `Std.Tui` facade over upstream `turbo-vision`. Treat this skill as project-local integration procedure, not as upstream API documentation.

## Required First Reads

Before edits, read these files in order:

1. `docs/pascal/std/tui/README.md`
2. `docs/pascal/std/tui/app/vm-bridge.md`
3. `docs/refactor/tui-bridge/00-context.md` when changing bridge architecture or modal/session behavior
4. `.agents/skills/fpas-change-checklist/SKILL.md` when implementing or modifying behavior, public API, docs under `docs/pascal/`, tests, compiler, VM, runtime, or stdlib code.

If the task is about examples of how to apply the skill, read `references/api_reference.md`.

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

## Migration Workflow

The Turbo Vision rewrite is complete. For new `Std.Tui` work:

1. Read the current spec under `docs/pascal/std/tui/`.
2. Follow the end-to-end recipe in `docs/pascal/std/tui/app/vm-bridge.md`.
3. Replace or extend tests that assert user-visible behavior, not deleted retained-engine internals.

## File and API Discipline

Follow project `AGENTS.md` structure rules:

- keep one concern per file;
- split large files before adding more mixed logic;
- prefer subdirectories over crowded modules;
- remove dead code introduced or exposed by the rewrite.

Expected VM bridge shape may change, but keep concerns separated:

```text
crates/fpas-vm/src/vm/execute/io/tui/
  session_app.rs    — live turbo-vision Application (main worker only)
  tv_run.rs         — Run entry, desktop projection
  exec_dialog.rs    — ExecDialog on live session
  file_dialog.rs    — RunFileDialog on live session
  reconcile.rs      — desktop rebuild after FPAS mutations
  interactive_loop.rs — scripted test loop only
  controls.rs, dialogs.rs, navigation.rs, …
crates/fpas-vm/src/vm/worker.rs — live_turbo_vision_app field
```

Open bridge refactors: `docs/refactor/tui-bridge/` (see `done/` for completed items).

## Documentation Rules

- Update `docs/pascal/std/tui/` for implemented behavior only.
- Keep speculative plans in `docs/future/`.
- Update Rust `///` docs that link to `docs/pascal/std/tui/` paths.

## Testing Rules

The rewrite is incomplete until testability is proven.

Prefer:

- Rust tests for bridge invariants, invalid handles, callback re-entry, cleanup, and diagnostics.
- FPAS tests for public behavior: open app, create window/dialog, add button, dispatch command, dialog result, input state, menu command.
- Headless tests over manual terminal checks.

Do not preserve tests that only validate old retained-view internals such as `QuerySceneGraph`, retained clip state, `HostProcessNext` integer tags, or old frame-root query records.

## License Rule

MIT is compatible with this workspace's BSD-3-Clause license. When using `turbo-vision` as a dependency, keep dependency metadata and lockfile accurate. When copying or vendoring upstream code, preserve MIT license and copyright notices.
