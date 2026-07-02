# `Std.Tui` Turbo Vision Rewrite

Status: **migration and post-migration phases complete on branch `turbo-vision-4-rust`** (2026-07-02).
Implemented behavior is documented under `docs/pascal/std/tui/`. This directory keeps the decision
record, phase history, and **remaining work**.

## Goal

Replace the custom retained `Std.Tui` engine with an FPAS-native facade over the Rust crate
`turbo-vision` from `aovestdipaperino/turbo-vision-4-rust`.

The rewrite removed every previous retained `Application.Host*` **public** API. There is no
backward-compatibility requirement for this work.

## Reading order

1. [Decision record](01-decision-record.md) — why the rewrite exists.
2. [Inventory](02-inventory.md) — what was removed, rewritten, or kept.
3. [Target API](03-target-api.md) — original planning sketch (archival; see current spec for truth).
4. [Implementation phases](04-implementation-phases.md) — Phases 0–8 checklist (**complete**).
5. [Testing plan](05-testing-plan.md) — automated and manual verification.
6. [Agent handoff](06-agent-handoff.md) — rules for continuing after context loss.
7. [Post-migration improvements](07-post-migration-improvements.md) — **start here for open work**.

## What is done

| Track | Status |
| --- | --- |
| Phases 0–8 (dependency spike → verification) | complete |
| Post-migration Phases A–G (read-back, menus, live tree, docs, command map, test seam, input hooks) | complete |
| Runtime setters, chrome refresh, `Application.Checked`, query bounds | complete |

## What is still open

See the **Remaining work** table in [07-post-migration-improvements.md](07-post-migration-improvements.md):

- ListBox selection (blocked on upstream)
- Optional removal of the hosted canvas loop (`minimal_application.fpas`)
- Manual terminal checks
- Merge to main (repository decision)
