# TUI Hosted Canvas Loop Removal

Status: **decided**.

## Decision

Remove the `Std.Tui` hosted canvas loop.

`Std.Tui` is for Turbo Vision applications: windows, dialogs, controls, menus,
status lines, command callbacks, and modal flows.

Simple terminal applications with their own paint loop belong to `Std.Console`.
They should use raw mode, alternate screen, structured terminal events, mouse
and resize handling, and direct console drawing APIs.

`Std.Graph` remains the native window/pixel surface. Its hosted dispatch is a
separate API and is not part of this cleanup.

## Required changes

1. Move terminal custom-canvas behavior out of `Std.Tui`.
   Remove `Application.Configure`, `ApplicationHandlers`, hosted canvas
   `Application.Run` dispatch, and the remaining private TUI `Host*` intrinsics
   that exist only for that path. **Done** — VM/sema/compiler/bytecode cleaned;
   `Application.Run` is Turbo Vision only.
2. Provide the needed `Std.Console` surface for simple interactive terminal
   apps. Prefer a small console event-loop helper only if it removes real
   duplication; otherwise use explicit `EnableRawMode`, `EnterAltScreen`,
   `ReadEventTimeout` / `PollEvent`, redraw flags, and cleanup in examples.
3. Rewrite `examples/math/mandelbrot/mandelbrot.fpasprj` so it imports
   `Std.Console` but not `Std.Tui`. Keep its terminal rendering, keyboard,
   mouse, resize, and cleanup behavior. **Done** — explicit raw-mode /
   alt-screen loop with `ReadEventTimeout`, `NeedsRedraw`, and inline handlers.
4. Keep `examples/math/mandelbrot/mandelbrot_graph.fpas` as the `Std.Graph`
   variant. Do not use it as a reason to keep the TUI hosted canvas loop.
5. Remove `examples/pascal/tui/minimal_application.fpas` or replace it with a
   true Turbo Vision facade example. **Done** — deleted; use existing Turbo
   Vision examples under `examples/pascal/tui/`.
6. Remove hosted-loop tests under `tests/tui/host/`, or move the relevant
   coverage to `tests/console/` if it now verifies `Std.Console` event-loop
   behavior. **Done** — `tests/tui/host/` removed; `tui_run_path_test.fpas`
   moved to `tests/tui/controls/`; screen asserts use `Std.Test` + `Std.Console`.
7. Update `docs/pascal/std/tui/` to describe only the Turbo Vision facade after
   the removal lands. **Done** — hub and `app/README.md` rewritten.
8. Update `docs/pascal/std/console/` if a new console loop helper or documented
   Mandelbrot pattern is added. **Done** — interactive loop section in console hub;
   remaining TUI subpages (`handlers`, `types`, `testing`, `vm-bridge`, `session`,
   `terminal-checklist`, `cell-width`) and `Std.Test` docs updated.

## Merge note

Before merging the Turbo Vision branch, remove
`docs/future/turbo-vision-4-rust/`. That directory is branch-local migration
history, including this decision note. If the hosted canvas loop removal is not
implemented before merge, move this decision back to a standalone
`docs/future/` document first.
