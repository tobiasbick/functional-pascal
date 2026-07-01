# `Std.Tui` Turbo Vision Rewrite

Status: **migration complete on branch `turbo-vision-4-rust`**. Current user-facing behavior is documented under `docs/pascal/std/tui/`. This directory keeps the decision record, inventory, and phase history.

## Goal

Replace the custom retained `Std.Tui` engine with an FPAS-native facade over the Rust crate `turbo-vision` from `aovestdipaperino/turbo-vision-4-rust`.

The rewrite broke every previous retained `Application.Host*` API. The project has no backward-compatibility requirement for this work.

## Reading Order

1. [Decision record](01-decision-record.md) — why the rewrite exists and what is in scope.
2. [Inventory](02-inventory.md) — code, docs, tests, and examples affected by removal.
3. [Target API](03-target-api.md) — proposed FPAS-facing API shape.
4. [Implementation phases](04-implementation-phases.md) — tracked checklist (Phases 0–8 and Phase 5 widgets complete).
5. [Testing plan](05-testing-plan.md) — headless tests, event injection, and verification.
6. [Agent handoff](06-agent-handoff.md) — rules for continuing after context loss.
7. [Post-migration improvements](07-post-migration-improvements.md) — planned follow-up phases (read-back, menus, live tree, …).

## Active Plan

[Implementation phases](04-implementation-phases.md) are complete. Active follow-up:

- [Post-migration improvements](07-post-migration-improvements.md) — one phase per change; start with Phase A (`ExecDialog` + field read-back).
- Manual terminal checks from [testing plan](05-testing-plan.md).
- Merge `turbo-vision-4-rust` when ready.
