# Std.Tui rewrite (try 2)

Planning documents for replacing the current FPAS Turbo Vision facade with a **Rust-owned widget tree** and a **Turbo-Vision-aligned Pascal API**.

This is a **breaking rewrite**. There is no backward-compatibility requirement for the hobby project. When implementation is complete, the public spec moves from here into `docs/pascal/std/tui/` and this directory is removed or archived.

## Status

| Item | State |
| --- | --- |
| Decision | Approved direction — Rust `turbo_vision::Application` is the single source of truth |
| Branch | `refactor/tui-try-2` |
| Implementation | Phase 1 complete; phase 2 vertical slice in progress (`try2/` on `refactor/tui-try-2`) |
| Smoke tests | `tests/tui/smoke/modal_button_try2_test.fpas`, `run_quit_try2_test.fpas` |
| Baseline | [baseline.md](baseline.md) — try-1 snapshot before rewrite |
| Upstream pin | `turbo-vision` 2.0, git tag `v2.0.0` (see workspace `Cargo.toml`) |

## Problem in one sentence

The current bridge keeps a parallel FPAS widget snapshot and rebuilds the live Turbo Vision desktop on every structural change (~6.5k LOC, 41 modules). That complexity is optional once Pascal only holds opaque view ids and every mutation goes straight to upstream.

## Target in one sentence

Pascal programs compose UI like upstream Turbo Vision (`Dialog.New`, `Add`, `ExecView`, `Run`), while the VM owns one live `turbo_vision::Application` and maps handles to upstream `ViewId` values — no reconcile, no `Bridged*` views, no command offset band.

## Document map

| Document | Contents |
| --- | --- |
| [Baseline snapshot](baseline.md) | Frozen try-1 metrics, API, tests, upstream `test_util` inventory |
| [Implementation status](status.md) | Living progress log and blockers |
| [Goals and principles](goals-and-principles.md) | Success criteria, constraints, explicit non-goals |
| [Current problems](current-problems.md) | Why the existing facade is hard to maintain |
| [Target architecture](target-architecture.md) | Runtime data flow, session model, headless path |
| [Target API](target-api.md) | Public Pascal types, record methods, examples |
| [Upstream mapping](upstream-mapping.md) | `turbo-vision` Rust API ↔ planned FPAS symbols |
| [Rust layout](rust-layout.md) | Crates, modules to add, modules to delete |
| [Migration phases](migration-phases.md) | Ordered implementation phases with exit criteria |
| [Testing strategy](testing-strategy.md) | FPAS tests, Rust tests, headless approach |
| [IDE migration](ide-migration.md) | `apps/ide` rewrite notes |
| [Deletion checklist](deletion-checklist.md) | Old symbols, files, and docs to remove |
| [Verification](verification.md) | Definition of done before deleting this plan |

## Related reading

- Current spec (to be replaced): [`docs/pascal/std/tui/`](../pascal/std/tui/)
- Current VM bridge notes: [`docs/pascal/std/tui/app/vm-bridge.md`](../pascal/std/tui/app/vm-bridge.md)
- Graph hosted dispatch (pattern reference): [`docs/pascal/std/graph/app/README.md`](../pascal/std/graph/app/README.md)
- Project structure rules: [`AGENTS.md`](../../AGENTS.md)
- Turbo Vision integration skill: [`.agents/skills/turbo-vision-4-rust/SKILL.md`](../../.agents/skills/turbo-vision-4-rust/SKILL.md)

## Quick comparison

| Aspect | Current (try 1) | Target (try 2) |
| --- | --- | --- |
| Widget authority | FPAS `TurboVisionObject` snapshot | Live `turbo_vision::Application` |
| Composition API | `Application.Create*` + `Application.AddChild(App, …)` | `Dialog.New` + `Dialog.Add` (record methods) |
| Structural updates | Full desktop rebuild (`reconcile.rs`) | Direct `Group::add` / remove on live tree |
| Command ids | Subset `Command.*` + `0x8000` offset band | Full upstream `CM_*` constants |
| Bridge size | ~6.5k LOC, 41 files | ~1.5–2.5k LOC, ~12–15 files (estimate) |
| Per-view `handle_event` | Not supported | Still not supported (language limit) |
