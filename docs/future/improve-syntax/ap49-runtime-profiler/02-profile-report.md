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
atomically, also after a runtime failure. Top-level fields:

| Field | Content |
| --- | --- |
| `format`, `version` | `fpas-profile`, `1` |
| `status`, `exit_code` | `ok` or `error`, and the process exit code |
| `wall_ms`, `instructions` | Whole-run wall time and dispatched instructions |
| `attribution` | Text describing batch-boundary attribution and its limits |
| `routines` | Top 25 by self time: name, source, line, self/inclusive instructions, percent and milliseconds |
| `lines` | Top 25 source lines by time, with routine name |
| `counters` | Every AP49.1 counter by name |
| `findings` | Short ordered statements derived from counters and samples |

Sources are project-relative paths when known. Milliseconds are rounded to three
decimals. Rows with zero instructions are omitted.

## Acceptance

VM tests check attribution to a hot callee, inclusive totals for its caller and
merging of pool-task samples. CLI tests check option validation, report shape,
publication after failure and that stdout is unchanged.
