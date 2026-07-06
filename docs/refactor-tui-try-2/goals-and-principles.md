# Goals and principles

## Primary goals

1. **Single source of truth** — The live `turbo_vision::Application` on the main worker owns the widget tree. Pascal values are opaque handles (`ViewId`) plus plain data records (`Rect`, menu items, etc.).

2. **Turbo-Vision-aligned ergonomics** — Programs read like upstream examples: construct views, attach children, run modally or on the desktop, handle commands at application level.

3. **Minimal bridge** — One concern per file, usually under 400 LOC. No snapshot/reconcile layer, no `Bridged*` adapter views unless upstream lacks a needed hook (prefer upstream APIs first).

4. **Testability without bespoke internals** — Headless tests drive upstream via `test-util` and `put_event` where possible; FPAS keeps thin test helpers only where upstream has no equivalent.

5. **Easier upstream bumps** — Pin `turbo-vision` by git tag; mapping table in [upstream-mapping.md](upstream-mapping.md) is the bump checklist.

## Success criteria

The rewrite is complete when all of the following hold:

- [ ] `apps/ide` builds and its test suite passes on the new API.
- [ ] All `tests/tui/` coverage is rewritten or replaced; no tests assert reconcile or `Bridged*` behavior.
- [ ] VM bridge under `crates/fpas-vm/src/vm/execute/io/tui/` is at most ~15 focused modules (see [rust-layout.md](rust-layout.md)).
- [ ] No `TurboVisionObject` enum, no `pending_reconcile`, no `command_map` offset band.
- [ ] Public docs live under `docs/pascal/std/tui/`; this plan directory is obsolete.
- [ ] Verification commands in [verification.md](verification.md) pass.

## Design principles (from AGENTS.md)

- **One concern per file** — split by widget family or bridge concern, not by “everything controls.”
- **Prefer subdirectories** — `views/` under the TUI bridge for per-widget lowering.
- **Reuse upstream** — call `Dialog::execute`, `message_box`, `FileDialog`, `Group::add`; do not reimplement TV behavior in FPAS.
- **No speculative API** — document only implemented symbols in `docs/pascal/`.
- **English** — all identifiers, comments, and user-facing docs.

## Explicit non-goals

| Non-goal | Reason |
| --- | --- |
| Literal Rust API in Pascal (`Box<dyn View>`, traits, `&mut self`) | FPAS has no traits, heap polymorphism, or borrow semantics |
| Borland C++ API compatibility (`TView*`, virtual methods, owner pointers) | Same language limits |
| Per-widget `handle_event` overrides in Pascal | Requires traits or a class system; use `OnCommand` / optional `OnKey` / `OnMouse` at app level |
| Custom retained scene graph / paint engine | Superseded by upstream Turbo Vision |
| Backward compatibility with try-1 `Application.Create*` API | Hobby project; break cleanly |
| Exposing every upstream widget in v1 | Ship core set first; add widgets incrementally via [upstream-mapping.md](upstream-mapping.md) |
| Precompiled `.fpaslib` or separate TUI link step | Out of project scope per AGENTS.md |

## Language-level adaptations (fixed)

These four differences from upstream Rust are **intentional** and permanent:

1. **Handles instead of ownership** — `ViewId` inside record types; Rust keeps `Box<dyn View>` internally.
2. **Record methods instead of `&mut self`** — `Dialog.Add(Self, Child)` maps to `group.add(Box::new(...))`.
3. **Application-level events** — `OnCommand` / `OnKey` / `OnMouse` callbacks instead of per-type `handle_event`.
4. **FPAS error model** — `Application.New` returns `Application` or uses runtime errors for terminal init failure (match existing `Std.Tui` session style).

## Optional enhancement (phase 2+)

**Hosted dispatch** like `Std.Graph` — `Application.Configure(App, Handlers)` with an `ApplicationHandlers` record. Not required for the first vertical slice; see [target-api.md](target-api.md#optional-hosted-dispatch).

## Decision log

| Date | Decision |
| --- | --- |
| 2026-07-06 | Abandon dual-state reconcile architecture; Rust-owned tree |
| 2026-07-06 | Widget API via typed records + `New`/`Add` methods, not `Application.Create*` namespace |
| 2026-07-06 | Expose upstream `CM_*` constants directly; drop `Command.*` subset and offset band |
| 2026-07-06 | Plan lives in `docs/refactor-tui-try-2/` until implementation lands in `docs/pascal/` |
