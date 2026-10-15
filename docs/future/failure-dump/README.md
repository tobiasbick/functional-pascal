# D1: automatic failure dump

Temporary implementation plan for [agent wish D1](../agent-wishes.md#d1-automatic-failure-dump).
The user approved the full D1 scope on 2026-10-15. This is a separate tooling
project, outside the AP campaign. Shared inspection, original-failure capture
and cooperative task collection, CLI integration and report publication are
implemented, independently reviewed and verified. Step 06's functional verification
and reproducible local measurement path are implemented, reviewed and verified.
Performance validation
remains open: the user explicitly excluded cloud timings as performance evidence.

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
| [x] | [02: shared inspection](02-shared-inspection.md) | 01 | Implemented and verified; existing TCP test failures documented |
| [x] | [03: failure capture](03-failure-capture.md) | 02 | Implemented and independently reviewed; workspace passes with two existing TCP tests excluded |
| [x] | [04: task snapshots](04-task-snapshots.md) | 03 | Implemented, independently reviewed and verified; workspace and final VM checks pass with two existing TCP tests excluded |
| [x] | [05: CLI and JSON report](05-cli-and-json-report.md) | 03, 04 | Implemented, independently reviewed and verified; real CLI/publication regressions and workspace checks pass with the same two existing TCP exclusions |
| [ ] | [06: verification and documentation](06-verification-and-documentation.md) | 02–05 | Functional verification, documentation and local measurement tooling complete and independently reviewed; valid local performance evidence remains open |

Next: **06, remaining local performance acceptance**. Functional checks,
documentation and the local measurement path are complete. Performance conclusions
require quiet local release runs; cloud smoke timings do not qualify.
The temporary plan stays until that remaining acceptance criterion is satisfied.

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

Step 01 was committed as `013500f2` with a Co-Author trailer. Step 02 is implemented,
verified and committed with a Co-Author trailer; Git history owns its commit identity.
Its [verification record](02-shared-inspection.md#verification-evidence) distinguishes
the existing TCP failures from the passing checks. Step 03 adds opt-in original-fault
capture and an embedded VM observer. It is implemented, independently reviewed,
verified and committed with a Co-Author trailer; Git history owns the commit
identity. Its [verification record](03-failure-capture.md#verification-evidence)
records the passing final workspace run and its explicit baseline exclusions.
Step 04 adds the bounded task catalog, cooperative peer frames and actual-root
globals. It is implemented, independently reviewed and verified; its
[verification record](04-task-snapshots.md#verification-evidence) owns the final
checks and measured development smoke sample. Step 05 adds the single opt-in
CLI option, bounded version-1 JSON and atomic fresh-file publication. It is
implemented, independently reviewed and verified; its
[verification record](05-cli-and-json-report.md#verification-evidence) records
real CLI regression proof and the passing workspace/final CLI checks. Step 06
adds the combined worker/JSON acceptance case, preserves original entry names
after `Wait`, and supplies the locally reproducible measurement workflow. Its
[verification record](06-verification-and-documentation.md#verification-evidence)
records the passing functional checks and independent review. Only valid local
performance acceptance remains open; D1 is not marked fully delivered yet.

## Removal after completion

When all six steps pass, move the implemented contract and examples into the
permanent CLI and debugger documentation, update the authoring reference and
skills as needed, and mark D1 implemented in `agent-wishes.md`. Remove this
entire directory and its planning links. Git history retains the decisions and
verification record; do not leave a second permanent specification here.
