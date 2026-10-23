# AP49: Runtime profiler for agents

Status: complete.
Effort: medium, across `fpas-bytecode`, `fpas-vm` and `fpas-cli`.
Completion is tracked only in the [central plan](../README.md#package-status).

## Goal

The profiler gives agents a one-shot profile report, designed like the
[batch debugger](../../../pascal/tools/debugger-batch.md): one run, one bounded
JSON document, stable field names, explicit units, structured findings with
their evidence and metadata for judging whether two reports compare.

## Current behavior and proposed contract

- Delivered contract: `fpas run --profile <path> <target> [-- args]` runs the
  program normally and then writes one JSON report to a new file. It contains
  wall time, dispatched instructions, the top routines (self and inclusive) and
  source lines, runtime event counters and findings. Without the option, event
  sites cost one relaxed atomic load and dispatch batches one pointer check.
  [AP49.4](04-agent-report-contract.md) delivered report version 2.
- Preserved behavior: program output, exit codes, diagnostics, scheduling and
  collection semantics are unchanged. The report never contains program values.
- Excluded scope: native Rust stack sampling, allocation-site tracking, a
  long-lived profiling protocol, editor integration, profiling `fpas test` or the
  debugger. Run comparison and scaling probes are follow-up candidates.

## Decisions

| ID | Question | Decision | Affected work packages |
| --- | --- | --- | --- |
| Q51 | Is an agent-oriented built-in profiler wanted, and who drives it? | User-approved; planning and implementation delegated to the agent on the AP48 branch, commits allowed, no push | AP49.1 through AP49.3 |
| Q52 | Which interface? | `run --profile <path>` writing one JSON report to a new file; program stdout and stderr stay untouched | AP49.2, AP49.3 |
| Q53 | How are routines and lines attributed? | At dispatch-batch boundaries: each batch's instructions and wall time go to the routine and line active when the batch ends; inclusive time adds every distinct routine on the call stack. Deterministic and cheap, with bounded attribution error per batch | AP49.2 |
| Q54 | Where do runtime counters live? | Process-wide, striped atomic counters in `fpas-bytecode`, enabled for one profiled run; VM and value modules report events there | AP49.1 |

Q52 to Q54 were chosen by the agent under the Q51 delegation. The collection
threshold amendment found while profiling was approved separately and is recorded
under Q42 in AP48.

## Open decisions

None.

## Dependencies and work packages

Prerequisites: none. AP48.1 through AP48.7 provide the collector and task
mechanisms whose events are counted.

- [x] [AP49.1: Runtime event counters](01-runtime-counters.md)
- [x] [AP49.2: Routine and line profile report](02-profile-report.md)
- [x] [AP49.3: Findings, documentation and dogfooding](03-findings-and-dogfooding.md)
- [x] [AP49.4: Reliable profile reports for agents](04-agent-report-contract.md)

This is the only work-package checklist for this AP. Follow
[status tracking](../development-process.md#status-tracking); completion does
not imply commit or merge approval.

## Acceptance

- `fpas run --profile report.json` on the headless Mandelbrot benchmark and on a
  synthetic live-graph program produces a valid report whose top routines,
  counters and findings identify the hot routine and the dominant runtime cost.
- Reports are written atomically to a new path; an existing path, `-` or an
  empty path is rejected before the program starts, like `--failure-dump`.
- A failing program still writes its report and keeps its exit code.
- Without `--profile`, VM benchmarks show no measurable regression.

## Documentation and regression owners

- Current documentation: [profile report](../../../pascal/tools/profiler.md),
  `fpas run --help`, the tools index.
- Implementation: `fpas-bytecode/src/profile.rs`, `fpas-vm/src/vm/profile/`,
  `fpas-cli/src/cli_profile/`.
- Tests: counter unit tests, VM attribution tests, CLI option and report tests.
- Cross-package checks: [shared constraints](../shared-constraints.md).
