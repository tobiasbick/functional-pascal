# 06: complete verification and permanent documentation

Depends on [02](02-shared-inspection.md), [03](03-failure-capture.md),
[04](04-task-snapshots.md) and [05](05-cli-and-json-report.md).
Progress lives in the [overview](README.md).

## Goal and deliverable

Verify the full D1 contract across the normal compiler/VM/CLI path, document
implemented behavior, and remove this temporary planning directory once its
scope is covered by permanent documentation and tests.

## Acceptance criteria

- One end-to-end failing program proves nested parameters/locals/captures,
  the true worker origin, the other task states and the produced JSON file.
  Separate focused cases cover lifecycle and unavailable-data behavior that
  the main acceptance case cannot reach.
- Repeated handled failures and retries release their retained snapshots.
  Capture/serialization failure does not hide the selected runtime diagnostic,
  leak coordination state or change normal task cleanup.
- Existing DAP/JSONL stack, scopes, values, task control and mutation remain
  covered by their current test owners. D1 does not remove either protocol.
- All relevant workspace and FPAS regression tests run, with passes, skips,
  unchanged baseline failures and unexecuted checks reported separately.
- Benchmarks compare the same code/toolchain baseline for scalar execution,
  calls, global access, task spawn/wait and task suspension. Measure capture
  disabled and enabled on successful runs, plus failure cost, peak report size
  and peer collection. Investigate reproducible regressions before completion;
  do not hide work by enabling deterministic/debug execution for a benchmark.
- The documented CLI examples and JSON fragments match actual behavior and
  parse successfully. Documentation-example tests and changed links pass.

## Permanent documentation owners

Update the implemented CLI invocation, file/error lifecycle and examples in
[CLI](../../pascal/program-structure/cli.md); document the full JSON schema,
bounds and observation semantics in a permanent page under `docs/pascal/tools/`.
Link it from the [tools index](../../pascal/tools/README.md),
[diagnostics](../../pascal/tools/diagnostics.md),
[debugger](../../pascal/tools/debugger.md) and main README where useful.

Check the [compact reference](../../pascal/agent-reference.md) using its
[maintenance guide](../../pascal/agent-reference-maintenance.md), and update
relevant authoring/debugging skills and AGENTS instructions only where their
workflow actually changes. Mark D1 implemented in [agent wishes](../agent-wishes.md).

Record measured validation before removing the plan. Delete this directory and
its links from the future index and wish list after permanent docs/tests own
the complete delivered scope. Do not mark D1 implemented while task inspection,
bounded missing-data reporting or publication verification is still unfinished.
