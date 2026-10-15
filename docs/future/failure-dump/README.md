# D1: automatic failure dump

Temporary implementation plan for [agent wish D1](../agent-wishes.md#d1-automatic-failure-dump).
The user approved the full D1 scope on 2026-10-15. This is a separate tooling
project, outside the AP campaign. No failure-dump behavior is implemented yet.

## Goal

An explicitly enabled `fpas run` writes one structured failure report containing
the original runtime diagnostic, the responsible task's call stack and frame
values, and the state and available frames of every relevant task. The ordinary
concurrent scheduler remains in use.

The feature is complete only when all work packages below are delivered and
verified. Completing the failing task's stack alone does not complete D1.
The [contract](01-contract.md) defines the bounds and honest representation of
unavailable data; these are part of the full implementation.

## Progress and implementation order

This is the only completion checklist. Individual files own their contract,
acceptance criteria and verification notes, without a second progress index.
Mark a work package complete only after its own acceptance criteria pass.
Record Git integration separately from implementation and verification.

| Done | Step | Depends on | Status |
| --- | --- | --- | --- |
| [x] | [01: contract](01-contract.md) | — | Defined; checked against the current runtime and CLI |
| [ ] | [02: shared inspection](02-shared-inspection.md) | 01 | Not started |
| [ ] | [03: failure capture](03-failure-capture.md) | 02 | Not started |
| [ ] | [04: task snapshots](04-task-snapshots.md) | 03 | Not started |
| [ ] | [05: CLI and JSON report](05-cli-and-json-report.md) | 03, 04 | Not started |
| [ ] | [06: verification and documentation](06-verification-and-documentation.md) | 02–05 | Not started |

Next: **02, shared read-only inspection**. Implementation tests belong to each
step; step 06 verifies the complete feature and its performance together.

## Boundaries

This work changes VM inspection and CLI tooling, without changing FPAS syntax,
evaluation, task ownership, failure selection, or Result handling. It adds no
debug session, deterministic scheduler, recording, replay, expression evaluation,
or debugger mutation. Existing DAP and JSONL protocols remain available.
Runtime diagnostic output remains separate from the dump document.

## Baseline and evidence

Inspection started at commit `853ed527`. The important existing boundaries are:

- [CLI execution](../../../crates/fpas-cli/src/cli_run.rs): `run_executable`
  receives only a diagnostic after `Vm::run` has finished cleanup.
- [Worker execution](../../../crates/fpas-vm/src/vm/worker/run.rs) and
  [pool execution](../../../crates/fpas-vm/src/vm/tasks/pool.rs): failure handling
  can unwind scopes and dispose of the worker before the CLI regains control.
- [Debugger inspection](../../../crates/fpas-vm/src/vm/debug/inspection): existing
  metadata, lexical visibility, reference resolution and value rendering.
- [Debugger contract](../../pascal/tools/debugger.md): all-stop debugging uses a
  different scheduler from normal execution, so it cannot implement ordinary
  `run` dumps by switching execution modes.

Step 01 is a documentation/design deliverable. No runtime test or implemented
CLI option is claimed by that completed row. Steps 02–06 remain open, and no
implementation commit exists for them.

## Removal after completion

When all six steps pass, move the implemented contract and examples into the
permanent CLI and debugger documentation, update the authoring reference and
skills as needed, and mark D1 implemented in `agent-wishes.md`. Remove this
entire directory and its planning links. Git history retains the decisions and
verification record; do not leave a second permanent specification here.
