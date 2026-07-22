# Phase 7 — Promote to `Std.Tui`

Execution rules and the current baseline: [implementation phases](../implementation-phases.md).
Until Gate 6.C passes, do not execute any Phase 7 task and do not document Tui3 behavior under
`docs/pascal/std/tui/`; that path still describes the production Turbo Vision facade.

## Task 7.1 — Freeze the exact migration manifest

**Status:** complete.

Turn Task 6.1's inventory into an exact rename/delete manifest. It must include:

- `Std.Tui` Pascal units, sema registration, compiler lowering, bytecode intrinsic variants, VM
  bridge modules, Rust tests, FPAS tests, examples, IDE usage, docs, Cargo dependency entries, and
  generated intrinsic completeness lists;
- all `Std.Tui2` units, tests, docs, examples, and manifest references;
- every `Std.Tui3` source/doc/test path to rename;
- `docs/future/tui-bridged-readback.md`, `docs/future/README.md`, and agent skills that describe the
  bridge as current;
- pre- and post-migration `rg` queries with expected zero/non-zero results.

No deletion occurs in this task.

Completed in [promotion-manifest.md](../promotion-manifest.md). The explicitly retained
`apps/ide/` legacy source is excluded from migration checks and remains non-buildable.

## Gate 7.A — Approve destructive migration

**Status:** human gate.

**Prerequisite:** Task 7.1.

A human reviews and approves the exact manifest. Approval covers only listed targets; newly found
targets return to Task 7.1.

## Task 7.2 — Remove the Turbo Vision `Std.Tui` implementation

**Status:** complete.

Delete exactly the approved production facade and VM bridge files, their dedicated tests/docs, and
the `turbo-vision` dependency only if the manifest proves no remaining consumer. Remove all related
intrinsic enum/decoder/completeness entries in the same change. Do not leave compatibility aliases.

The only recovered common code is the hosted callback-record validation now owned by
`crates/fpas-vm/src/vm/execute/io/hosted_common.rs`, because `Std.Graph` uses it. The legacy
`apps/ide/` source remains explicitly excluded and non-buildable.

## Task 7.3 — Remove `Std.Tui2`

**Status:** ready.

Delete exactly the approved Tui2 units, tests, docs, examples, and manifest rows. Verify the
approved reference queries before continuing.

## Task 7.4 — Rename Tui3 to `Std.Tui`

**Status:** blocked by Task 7.3.

Apply the approved path, unit, import, documentation, example, test, and manifest rename. Use the
final names only; do not retain `Tui3` aliases or migration terminology. Update Rust doc links.

## Task 7.5 — Remove obsolete future material and verify

**Status:** blocked by Task 7.4.

Delete `docs/future/tui3/`, the Tui2 freeze notice, and
[`tui-bridged-readback.md`](../../tui-bridged-readback.md); update
[`docs/future/README.md`](../../README.md), `AGENTS.md`, `AI_CONTRIBUTING.md`, and project skills. Run
all approved reference queries, the full common checkpoint, production application tests, and the
interactive terminal checklist rewritten for the promoted implementation.

## Phase completion

Only one public `Std.Tui` remains; no `Std.Tui2`, `Std.Tui3`, Turbo Vision bridge, obsolete
intrinsic, dependency, test, documentation, or skill reference remains.
