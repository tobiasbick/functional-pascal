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

## Current Status

- [x] Branch created: `turbo-vision-4-rust`.
- [x] Upstream checked: `turbo-vision` 1.3.1, Rust 2024, MIT license, `crossterm` 0.29.
- [x] Local compatibility checked: this workspace already uses `crossterm` 0.29.
- [ ] Phase 1 API design accepted in code.
- [ ] Minimal spike implemented.
- [ ] Old retained TUI engine removed.
- [ ] New `docs/pascal/std/tui/` spec written after implementation.

## Go/No-Go Gate

Do not start broad deletion until the minimal spike proves all of these:

- FPAS can create a Turbo Vision application.
- FPAS can create at least one window/dialog and one button.
- A Turbo Vision command can call back into FPAS.
- FPAS can request application shutdown from a command handler.
- The flow can be tested without manual terminal interaction.

If the callback or testability gate fails, stop and update [implementation phases](04-implementation-phases.md) with the blocker before deleting old code.
