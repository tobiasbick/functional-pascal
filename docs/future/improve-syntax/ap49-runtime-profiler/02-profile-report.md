# AP49.2: Routine and line profile report

Package: [AP49](README.md). Status: complete. Decisions: Q52 and Q53.
Prerequisite: [AP49.1](01-runtime-counters.md).

## VM sampling

`Vm::enable_profile()` installs a sink in the hosted state. Each worker lazily
creates a local buffer and, after every dispatch batch, adds the batch's
instructions and wall time to the current instruction address (self) and to each
distinct function on its call stack (inclusive). Buffers merge into the sink when
the worker is dropped, so task slices on other threads are included. After the
run, `Vm::profile_report()` resolves addresses through the executable's source
map and sequence points into routine and line rows. Self samples are keyed by the
worker's current function and address, so a batch ending on a callee's `Return`
counts for the caller, consistent with its inclusive stack.

## Report

`fpas run --profile <path>` validates the destination like `--failure-dump`,
enables counters and VM sampling, runs the program and publishes the report
atomically, also after a runtime failure. AP49.2 delivered report version 1;
[AP49.4](04-agent-report-contract.md) replaced it with version 2, documented in
the [handbook](../../../pascal/tools/profiler.md).

## Acceptance

VM tests check attribution to a hot callee, inclusive totals for its caller and
merging of pool-task samples. CLI tests check option validation, report shape,
publication after failure and that stdout is unchanged.
