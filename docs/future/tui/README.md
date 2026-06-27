# TUI roadmap

This directory is the planning home for future `Std.Tui` work. Current user-facing behavior is
specified under [`docs/pascal/std/tui/`](../../pascal/std/tui/); this directory is only for completed
implementation records and new implementation plans.

## Current Status

The retained TUI foundation is implemented:

- Rust-hosted `Application.Run` with `On*` handlers and headless testing.
- Retained views with focus, modal scope, local painting, clipping, damage, pointer capture, and
  sourced commands.
- Frame roots with Turbo Vision-style chrome, scrolling, close/zoom, move/resize, activation,
  cascade/tile, and owned framed dialogs.
- Standard retained controls: label, button, input line, checkbox, radio group, list box, scroll
  bar, scroll view, memo, shared scroll model, Unicode cell-width handling, and anchor/grow layout.

The compact completion record is [`completed.md`](completed.md).

## Implementation Plans

Work these in order unless a bug requires a narrower fix first.

| Order | Plan | Status | Purpose |
| --- | --- | --- | --- |
| 1 | [`plan-quality-tooling.md`](plan-quality-tooling.md) | Next | Strengthen verification, property tests, real-terminal smoke coverage, and doc-link checks. |
| 2 | [`plan-widget-polish.md`](plan-widget-polish.md) | Planned | Improve retained widget ergonomics without changing the core architecture. |

## Rules for New TUI Plans

- Keep completed history in [`completed.md`](completed.md), not in active plan files.
- Each plan must have checkboxes, acceptance criteria, docs impact, and verification commands.
- Move a plan item to `completed.md` when it is implemented and verified.
- Do not describe planned behavior under `docs/pascal/`; current specs only document shipped
  behavior.

## Current Specs

- [`docs/pascal/std/tui/app/README.md`](../../pascal/std/tui/app/README.md)
- [`docs/pascal/std/tui/app/frames.md`](../../pascal/std/tui/app/frames.md)
- [`docs/pascal/std/tui/app/controls.md`](../../pascal/std/tui/app/controls.md)
- [`docs/pascal/std/tui/app/views.md`](../../pascal/std/tui/app/views.md)
- [`docs/pascal/std/tui/app/handlers.md`](../../pascal/std/tui/app/handlers.md)
- [`docs/pascal/std/tui/cell-width.md`](../../pascal/std/tui/cell-width.md)
