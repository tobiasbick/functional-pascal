# Audit Remediation Plan

Created from AGENTS.md / docs / examples audit (2026-06-07).

Workflow per item: implement → test → update this file → commit → wait.

## P0 — Documentation correctness (user-facing)

| # | Status | Task |
|---|--------|------|
| 1 | done | `docs/pascal/11-stdlib.md` — remove poll-style `Application.ReadEvent` from `Std.Tui` row |
| 2 | done | `docs/pascal/std/tui-app.md` — `Application.Run` prerequisites: include host widget views |
| 3 | done | `docs/rust/parallel-vm.md` — fix `tui.rs` path, intrinsic range, test paths |
| 4 | done | `docs/pascal/std/tui-app.md` — remove or mark `OnStartup` as non-shipped |

## P1 — Complete TUI/menu spec

| # | Status | Task |
|---|--------|------|
| 5 | done | `tui-app.md` — add process tag 21 |
| 6 | done | `tui-app.md` — add intrinsics 343–347 to Pascal names + VM bridge tables |
| 7 | done | `tui-app.md` — document menu input priority and navigation |
| 8 | done | `docs/pascal/std/README.md` — extend intrinsic range to 256–347 |
| 9 | pending | `docs/future/README.md` — fix TUI poll claim; move Json/Parse to implemented |
| 10 | pending | `docs/pascal/11-stdlib.md` — add `Std.Proc` and `Std.Json` rows |

## P2 — Structure (AGENTS.md)

| # | Status | Task |
|---|--------|------|
| 11 | pending | Split `menu_bar.rs` (716 LOC) |
| 12 | pending | Split `graph/tests.rs` (816 LOC) |
| 13 | pending | Consolidate TUI siblings under `fpas-std/src/tui/`; relocate `helpers.rs` |
| 14 | pending | Break `menu_bar`/`menu_popup` circular dependency |

## P3 — Examples & CI

| # | Status | Task |
|---|--------|------|
| 15 | pending | Re-enable graph compiler tests (stale `#[ignore]` reasons) |
| 16 | pending | Add `host_dispatch_*.fpas` + `apps/ide/ide.fpasprj` to CI allowlist |
| 17 | pending | Add `examples/pascal/tui/menu_bar.fpas` |
| 18 | pending | Document `apps/ide/` in examples README |
| 19 | pending | Clarify `apps/portal/` as illustrative in `10-projects.md` |
