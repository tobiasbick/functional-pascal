# 06: complete verification and permanent documentation

Depends on [02](02-shared-inspection.md), [03](03-failure-capture.md),
[04](04-task-snapshots.md) and [05](05-cli-and-json-report.md).
Progress lives in the [overview](README.md).

## Goal and deliverable

Verify the full D1 contract across the normal compiler/VM/CLI path, document
implemented behavior, and remove this temporary planning directory once its
scope is covered by permanent documentation and tests.

On 2026-10-15 the user explicitly excluded cloud timing results as valid
performance evidence. Complete functional verification and measurement tooling
here, but leave the performance criterion and final plan removal open until
quiet local release measurements are available. Do not record cloud samples in
benchmark history or infer an overhead/zero-cost claim from them.

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

## Functional verification and local measurement preparation

The real CLI acceptance case now starts a worker through a tail-called entry,
fails inside nested routines, and checks parameter/local/capture/global values,
the original non-main task ID, the main propagation entry and the published JSON.
It exposed a missing task entry name after `Wait` released the scheduler record.
The original entry ID is now retained with the same bounded diagnostic-owned
snapshot. The regression failed before repair and passes afterward, including
the distinction between a tail-called entry name and the oldest retained frame.

Existing VM tests own all saved/live/terminal task states, the native collection
deadline, cleanup ordering, handled failures, retries and release of retained
storage. Existing debugger tests continue to own DAP/JSONL control and mutation.
The new [FPAS benchmark driver](../../../examples/pascal/failure-dump/failure-dump-benchmark.fpas)
pairs identical compiled images with capture disabled/enabled for scalar work,
calls, globals, spawn/wait, timer suspension, small failures and large reports.
The [runner smoke test](../../../tests/runner/failure_dump_benchmark_test.fpas)
executes every case without treating its timings as performance evidence.

The permanent [local measurement guide](../../bench/failure-dump.md) defines
timers, warmup, alternating sample order, raw samples, report size checks and
reproduction commands. Runtime-report documentation is linked from the main
README, debugger and FPAS authoring skill. The compact reference and AGENTS
instructions already describe the implemented workflow accurately.

## Verification evidence

- `cargo fmt --all -- --check`, `cargo build` and the complete workspace tests
  pass. The workspace includes 586 CLI unit tests, 647 VM unit tests, 14 CLI
  failure-report tests, FPAS suites, handbook examples, DAP/JSONL tests, other
  packages and doctests. Only the two unchanged TCP tests documented in
  [step 02](02-shared-inspection.md#verification-evidence) are excluded:
  `tcp_backpressure_can_time_out_and_cancel_without_closing` and
  `write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts`.
- The new CLI regression failed before the task-entry repair for the intended
  missing-metadata reason. It now passes with the original name despite the
  entry frame's tail-call removal. Existing handled/retry tests still prove
  release of all charged snapshot storage. Directly ignored native-process
  fixtures are executed by their passing parents; existing intentional FPAS
  runner skips remain unchanged.
- `fpas test --std-lib lib --file tests/runner/failure_dump_benchmark_test.fpas
  tests/suite.fpasprj` passes. It executes all seven workload scenarios at one
  measured pair for correctness only. The full workspace also executes this
  test through its existing FPAS runner suite.
- `fpas fmt --check examples/pascal/failure-dump
  tests/runner/failure_dump_benchmark_test.fpas` passes. The existing compact
  reference test and documentation-example checker pass in the workspace run.
  The complete documented JSON parses with `Std.Json` and matches a freshly
  generated report from the current CLI.
- Independent subagent review found no code/test blocker and one missing
  benchmark-group entry in the handbook's suite-editing list; it is corrected.
  All 173 relative file links in changed Markdown resolve. The FPAS authoring
  skill's unchanged two-field frontmatter is valid, and `git diff --check` passes.
- No release performance run or benchmark-history update was performed in
  this cloud environment. Those remain unexecuted, not passed or inferred from
  smoke-test timings. This directory stays until local performance acceptance.

Workspace command:

```text
cargo test --workspace -- --skip tcp_backpressure_can_time_out_and_cancel_without_closing --skip write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts
```

## Remaining local acceptance

- Run the `failure-dump` group with the same checkout/toolchain on a quiet local
  release build; inspect all disabled/enabled sample arrays and medians, repeat
  noise, investigate reproducible regressions and record settled local evidence.
- Profile live-peer collection separately from mandatory semantic owner cleanup;
  the paired CLI workloads do not isolate that boundary. Retain the existing
  blocked-native deadline/early-notification tests as functional proof.
- After performance acceptance, update the overview and wish status, preserve
  settled results in permanent benchmark documentation/history as appropriate,
  and remove this temporary directory and its planning links.
