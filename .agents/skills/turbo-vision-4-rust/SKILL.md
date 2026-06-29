---
name: turbo-vision-4-rust
description: Use when planning or implementing the Functional Pascal `Std.Tui` rewrite over the Rust `turbo-vision` crate from `aovestdipaperino/turbo-vision-4-rust`, especially tasks touching `docs/future/turbo-vision-4-rust`, `docs/pascal/std/tui`, TUI sema/compiler/bytecode/VM/runtime code, TUI tests, examples, or `apps/ide`. Also use when checking upstream Turbo Vision API, dependency, license, crossterm compatibility, or deciding whether to delete old `Application.Host*` retained-view APIs.
---

# Turbo Vision 4 Rust

Guide the `Std.Tui` rewrite toward a small FPAS-native facade over upstream `turbo-vision`. Treat this skill as project-local migration procedure, not as upstream API documentation.

## Required First Reads

Before edits, read these files in order:

1. `docs/future/turbo-vision-4-rust/README.md`
2. `docs/future/turbo-vision-4-rust/01-decision-record.md`
3. `docs/future/turbo-vision-4-rust/04-implementation-phases.md`
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

Follow `docs/future/turbo-vision-4-rust/04-implementation-phases.md`.

1. Keep the phase checklist current in the same change.
2. Prove the minimal dependency and callback spike before broad deletion.
3. Do not delete the old TUI engine until FPAS command callback and headless/test execution are proven.
4. After the spike, remove old retained-view internals instead of extending them.
5. Replace tests that assert deleted internals with tests for user-visible behavior.

Good first implementation target:

- add `turbo-vision = "1.3.1"` or the verified current version;
- build;
- inspect `cargo tree -i crossterm`;
- implement the smallest FPAS command callback spike;
- update phase status with commands run and outcomes.

## File and API Discipline

Follow project `AGENTS.md` structure rules:

- keep one concern per file;
- split large files before adding more mixed logic;
- prefer subdirectories over crowded modules;
- remove dead code introduced or exposed by the rewrite.

Expected VM bridge shape may change, but keep concerns separated:

```text
crates/fpas-vm/src/vm/execute/io/tui/
  application.rs
  callbacks.rs
  commands.rs
  controls.rs
  dialogs.rs
  events.rs
  handles.rs
  testing.rs
```

## Documentation Rules

- Keep future plans in `docs/future/turbo-vision-4-rust/`.
- Update `docs/pascal/std/tui/` only for implemented behavior.
- Delete or rewrite stale `docs/pascal/std/tui/app/*` pages when their old Host API is removed.
- Update Rust `///` docs that link to old TUI paths.

## Testing Rules

The rewrite is incomplete until testability is proven.

Prefer:

- Rust tests for bridge invariants, invalid handles, callback re-entry, cleanup, and diagnostics.
- FPAS tests for public behavior: open app, create window/dialog, add button, dispatch command, dialog result, input state, menu command.
- Headless tests over manual terminal checks.

Do not preserve tests that only validate old retained-view internals such as `QuerySceneGraph`, retained clip state, `HostProcessNext` integer tags, or old frame-root query records.

## License Rule

MIT is compatible with this workspace's BSD-3-Clause license. When using `turbo-vision` as a dependency, keep dependency metadata and lockfile accurate. When copying or vendoring upstream code, preserve MIT license and copyright notices.
