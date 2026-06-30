# `Std.Tui` Turbo Vision Rewrite

Status: planned. This directory describes a future rewrite only. Do not treat these files as the current `Std.Tui` specification.

Current user-facing `Std.Tui` behavior remains documented under `docs/pascal/std/tui/` until the rewrite is implemented. Planned or speculative behavior belongs here, not in `docs/pascal/`.

## Goal

Replace the custom retained `Std.Tui` engine with an FPAS-native facade over the Rust crate `turbo-vision` from `aovestdipaperino/turbo-vision-4-rust`.

The rewrite may break every current `Std.Tui` API. The project has no backward-compatibility requirement for this work.

## Reading Order

1. [Decision record](01-decision-record.md) - why the rewrite exists and what is in scope.
2. [Inventory](02-inventory.md) - current code, docs, tests, and examples affected by removal.
3. [Target API](03-target-api.md) - proposed FPAS-facing API shape.
4. [Implementation phases](04-implementation-phases.md) - tracked checklist for execution.
5. [Testing plan](05-testing-plan.md) - headless tests, event injection, and verification.
6. [Agent handoff](06-agent-handoff.md) - rules for continuing after context loss.

## Active Plan

Use [implementation phases](04-implementation-phases.md) as the tracked checklist and handoff point.

Planned next work:

- Add old-symbol diagnostics that point users at the current Turbo Vision facade.
- Replace retained-engine internals with Turbo Vision-backed application, dialog, command, event, and widget modules.
- Migrate TUI examples and `apps/ide` to the new API.
