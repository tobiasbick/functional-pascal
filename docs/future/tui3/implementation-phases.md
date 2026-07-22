# Std.Tui3 implementation phases

## How to execute this plan

This file is the execution checklist. The other files in this directory define the contracts.
An implementation task must not invent a different API or silently broaden its scope. If a task
conflicts with a linked contract, stop and update the plan before changing code.

Task status starts with one of these values and may name its prerequisite:

- `complete` — implemented and covered by the named regression tests;
- `ready` — contract and scope are fixed; an implementation agent may execute it;
- `blocked` — a named prerequisite is missing;
- `architecture gate` — requires an explicit design decision before implementation;
- `human gate` — requires an explicit promotion or destructive-change approval.

For every `ready` task:

1. Touch only the listed files and directly required facade/test manifests. Stop if another compiler,
   VM, or public-unit layer becomes necessary and add that scope to this plan first.
2. Reuse the named Tui2 implementation for algorithms only. Do not copy retained handles, parent
   pointers, registries, callbacks, invalidation state, or `Create`/`Add`/`Destroy` APIs.
3. Add the named tests in the same change and bundle new FPAS tests through
   [`tests/suite.fpasprj`](../../../tests/suite.fpasprj).
4. Update implemented behavior under `docs/pascal/std/tui3/`; leave unimplemented behavior here.
5. Run targeted tests while developing. At a phase checkpoint run `cargo fmt`, `cargo build`,
   `cargo test --workspace`, `cargo run -q -p fpas-cli -- fmt --check tests/ examples/ apps/`, and
   `cargo run -q -p fpas-cli -- test tests/`.

## Current implementation baseline

Phase 0 produced more than a signature spike. The following code is the baseline and must be
extended rather than recreated:

| Area | Current implementation | Remaining owner |
| --- | --- | --- |
| Geometry | `Geometry/Point.fpas`, `Geometry/Size.fpas`, `Geometry/Rect.fpas` | Complete unless a later task exposes a regression. |
| Identity | `Ids/ControlId.fpas`, `Ids/Action.fpas` | Complete unless a later task exposes a regression. |
| Elements | Recursive `TuiElement` with Empty, Label, Button, Input, CheckBox, List, Scroll, MenuBar, StatusLine, Row, Column, Layout, Spacer, Window, Dialog, Desktop | Phase 4 complete. |
| Layout | Measure + private arranged-frame arrange in `Layout/`; host-owned `TuiArrangedFrame` | Phase 2 complete. Grid/Form/Stack remain deferred. |
| Rendering | Cell working surface; paint from arranged geometry only via clipped canvas; terminal-too-small overlay | Phase 4 complete. |
| Runtime | Headless host, injection, FIFO queue, key/pointer routing, focus helpers, ticks, resize, commands, snapshots | Phase 3–4 complete. Interactive terminal is Phase 5. |
| Tests | Phase 0–4 FPAS regressions under `tests/stdlib/tui3/` | Keep them as permanent canaries. |

The recursive `Children: array of TuiElement` representation is intentional. The compiler stack
overflow exposed by that representation was fixed in `fpas-sema`; do not replace the application
tree with public node ids or a retained parent registry. Layout may use a private, frame-scoped flat
index as described in [layout.md](layout.md).

## Cross-cutting language follow-ups

For every compiler panic or language restriction found while implementing Tui3, add an
entry to [compiler-panic-followups.md](../compiler-panic-followups.md) in the same change.
Do not silently change the language or leave an undocumented workaround.

## Phase 0 — Plans and executable feasibility

- Write and keep `docs/future/tui3/` current.
- Mark `docs/future/tui2/` frozen/superseded.
- Document implemented behavior under `docs/pascal/std/tui3/`; keep later work in this plan.
- **Complete — compile gate + headless confirm-dialog slice.** `Std.Tui3` exports
  `TuiApplication.RunIterations<TModel>` with
  `Update: function(State: TModel; Msg: TuiMsg; Cmd: TuiCmdOutput): TModel` and
  `View: function(State: TModel): TuiElement`. Covered by
  [`tests/stdlib/tui3/mvu_host_signature_test.fpas`](../../../tests/stdlib/tui3/mvu_host_signature_test.fpas)
  and
  [`tests/stdlib/tui3/confirm_dialog_slice_test.fpas`](../../../tests/stdlib/tui3/confirm_dialog_slice_test.fpas),
  plus
  [`tests/stdlib/tui3/element_tree_test.fpas`](../../../tests/stdlib/tui3/element_tree_test.fpas)
  (label/input/button/window/dialog, routed focus/TextChanged/Action/Quit, snapshots, unique control
  ids, and nested child structure). Negative runtime canaries reject forged and duplicate ids.
  Naming note: FPAS reserves `None` as a token, so v1 uses `TuiCmd.NoCommand` and
  `TuiElementBuilders.MakeEmpty`.
  Representation note: `TuiElement` uses recursive `Children: array of TuiElement` storage. The
  generic-inference stack overflow found by the spike is fixed in `fpas-sema` and covered by a Sema
  regression plus the MVU host test. `TuiMsg` enum payloads use `TuiControlId` and `TuiAction`
  directly; imported record types in associated fields are covered by a linker regression. FPAS
  mutable value parameters are local bindings, so the compiling command output is the reusable
  host-owned `TuiCmdOutput` capability rather than the original ineffective mutable enum parameter.
  [`tests/stdlib/tui3/repeated_frames_test.fpas`](../../../tests/stdlib/tui3/repeated_frames_test.fpas)
  covers 100 frames each at 40×12 and 120×40; [testing.md](testing.md) records the baseline and
  clone-ownership evidence.
- If aggregate cloning dominates, implement shared or copy-on-write VM storage as a prerequisite and
  repeat the spike. Do not introduce public retained view handles as a workaround.

Completion: the slice passes [testing.md](testing.md)'s Phase 0 suite and the plan records the final
compiling signatures. This gate is passed; Phase 1 is now unblocked.

## Remaining phases

Execute one task at a time from the phase files. Do not skip an architecture or human gate.

| Phase | Task file | Entry status |
| --- | --- | --- |
| 1 — Values and owned rendering storage | [Phase 1](phases/phase-1-values.md) | Phase 1 complete; Phase 2 complete. |
| 2 — Deterministic layout and arranged-frame paint | [Phase 2](phases/phase-2-layout.md) | Phase 2 complete; Phase 3 complete. |
| 3 — Headless MVU hardening | [Phase 3](phases/phase-3-headless-runtime.md) | Phase 3 complete; Phase 4 complete. |
| 4 — Controlled controls and application chrome | [Phase 4](phases/phase-4-controls.md) | Phase 4 complete; Phase 5 Gate 5.A is ready. |
| 5 — Interactive terminal | [Phase 5](phases/phase-5-terminal.md) | Phase 5 complete. |
| 6 — Production-readiness gate | [Phase 6](phases/phase-6-readiness.md) | Blocked by Phase 5. |
| 7 — Promote to `Std.Tui` | [Phase 7](phases/phase-7-promotion.md) | Blocked by promotion decision. |

## Explicit non-work

- Finishing remaining Tui2 retained controls for their own sake.
- Declarative wrappers over Tui2 handles.
- React-hooks component state as the primary model.
- Arbitrary closure commands or worker posting in v1.
- Claiming full Turbo Vision desktop behavior from static TV-looking chrome.
- Keeping three public TUI units after a successful promote.
