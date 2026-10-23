# AP49.4: Reliable profile reports for agents

Package: [AP49](README.md). Status: planned; documentation only is authorized.
Prerequisites: [AP49.2](02-profile-report.md) and
[AP49.3](03-findings-and-dogfooding.md), both delivered.

## Goal and current behavior

The profiler already supports a non-interactive run, versioned JSON, bounded
routine and line tables, source locations, counters and suggested next steps.
This work package closes gaps that can mislead an agent:

- `self_percent`, `inclusive_percent` and line `percent` describe instruction
  shares, while the tables rank by time. Their names do not identify the unit.
- The classification finding claims that every aggregate store revalidates the
  whole destination graph. Stores now validate only the stored value; a high
  visit count alone cannot establish the cause.
- The worker-idle finding assumes the hardware thread count minus one instead
  of the VM's actual pool size.
- The dominant-routine finding also fires on a program's entry routine for
  trivial loops, which adds noise rather than a lead.
- Findings are strings. Consumers must interpret prose to identify a rule,
  recover its evidence or associate it with a routine.
- Reports show neither how parallel a run was nor the executable identity and
  effective runtime configuration needed to judge whether two runs compare.

The contract below is proposed for implementation. Creating this plan does not
authorize source changes, commits or profiling runs.

## Order of work

The classification text and the worker count are defects in the delivered
version 1 report. Repair them first, with tests, as a separate change; they do
not depend on the version 2 contract. The remaining items replace the schema.

## Proposed report contract

Keep `fpas run --profile <new-file>` and publish report format version `2`.
Replace ambiguous fields directly; do not emit compatibility aliases or retain
a second renderer for version `1`.

### Explicit shares

Routine rows expose `self_time_percent` and `inclusive_time_percent`; line rows
expose `time_percent`. Keep the absolute instruction counts and millisecond
fields; instruction shares follow from the counts and the run total and are not
duplicated as percentages.

Time shares divide by `sampled_ms` using unrounded internal durations. A zero
denominator produces zero. Inclusive rows can overlap and their shares must not
be summed as a partition. Keep descending self-time ordering for routines and
time ordering for lines, with deterministic tie breakers. Expose the total
eligible row counts and the limit so agents can recognize omitted rows.

Add `parallelism`, the ratio of `sampled_ms` to `wall_ms`: the average number of
threads executing FPAS code. It identifies load imbalance and serial phases
without a separate finding.

### Findings with evidence

Replace each finding string with an object containing:

| Field | Proposed meaning |
| --- | --- |
| `code` | Stable rule identifier, independent of message wording |
| `message` | Short statement of the observed cost |
| `evidence` | Named numeric measurements, each with its unit and denominator where applicable |
| `location` | Routine, source and line when known; otherwise `null` |
| `next_steps` | Suggested checks in plain text, never commands executed by the profiler |

Use one object per triggered rule and location, in a documented deterministic
order. Bound the number of findings and text lengths. Evidence must come from
the same measurements used to trigger the rule. Keep the current rule thresholds
unless a separate, documented correctness repair is necessary.

For ownership classification, report `classify_calls`, `classify_visited` and
visits per call. Suggest varying the stored value's graph size and inspecting
the hot call sites. Do not assert whole-destination revalidation, a memory leak
or another unmeasured cause. Other findings must also separate observations
from hypotheses. For worker-idle ratios, use the actual pool size and measured
interval. The dominant-routine rule skips the program entry routine.

### Comparison metadata

Add a bounded `run` object with these fields:

| Field | Proposed meaning |
| --- | --- |
| `tool` | FPAS version and build revision when available; unknown revision is `null` |
| `target` | Target kind, project-relative identifier when available, and digest of the executed bytecode artifact including its executable metadata |
| `runtime` | Effective pool worker count and the collection settings that affect execution, including defaults |
| `sampling` | Attribution method, maximum batch size and report row limits |

These fields replace the version 1 `attribution` text; its explanation moves to
the handbook. Use one documented digest algorithm and compute it outside the
timed interval. Do not embed source text, program values, program arguments,
environment variables or complete command lines; omit absolute target paths.
Program arguments and the host environment are known to the invoking agent and
are not reported.

Document that matching metadata is necessary for a controlled comparison but
does not prove equal arguments, input files, network responses, external state,
machine load or power settings. Agents must control those conditions separately.
Comparing instrumented runs also requires the same profiler configuration;
their wall times do not establish the overhead of unprofiled execution.

### Counter scope

Counters are process-wide. Two VMs profiling concurrently in one process, as an
embedding host could arrange, would mix their counts. Either make the counters
VM-scoped or document the limit and set a report flag when more than one run
enabled counting at the same time. `fpas run` itself runs one VM and is not
affected.

## Preserved behavior and exclusions

Preserve program stdout, stderr, exit codes, atomic publication to a new file,
reports after runtime failures, sampling attribution and collection policy.
Document the existing batch-boundary attribution limits alongside the new fields.
The report remains a bounded summary; no interactive session is required.

This package does not add a report-comparison command, performance gates, scaling
probes, allocation-site tracking, editor integration or profiling of `fpas test`.
It does not change language syntax or authorize an AP48 collection-policy change.

## Implementation scope

Before implementation, inspect these owners and refine the file list only where
an effective setting or executable identity is unavailable at the CLI boundary:

| Path | Purpose |
| --- | --- |
| `crates/fpas-cli/src/cli_profile/mod.rs` | Version 2 rendering, explicit shares, parallelism, table coverage and run metadata |
| `crates/fpas-cli/src/cli_profile/findings.rs` | Structured findings, measured evidence and corrected explanations |
| `crates/fpas-cli/src/cli_profile/tests.rs` | Report contract and finding rules with controlled evidence |
| `crates/fpas-cli/tests/profile_report.rs` | Real CLI publication, metadata and output boundaries |
| `crates/fpas-cli/src/cli_run.rs` and `crates/fpas-vm/src/vm/profile.rs` | Expose the effective pool size and executable identity; locate actual setting owners before editing |
| `crates/fpas-bytecode/src/profile.rs` | Counter scope: VM-scoped storage or concurrent-run detection |
| `docs/pascal/tools/profiler.md` | Implemented version 2 schema, denominators, attribution, metadata limits and agent workflow |
| `crates/fpas-cli/src/cli_input/help.rs` | Confirm the example and option description still match |
| AP49 planning files and the central plan | Reflect the delivered contract and close AP49.4 after verification |

Search report consumers and migrate every version 1 field reference, including
fixtures and documentation examples. Keep the existing CLI example unless the
implemented interface requires a change. Planned behavior stays in this file
until implementation.

## Acceptance and verification

1. A controlled report where time and instruction shares differ exposes the
   named time shares and the instruction counts. Zero totals are finite zeros;
   inclusive overlap is documented; ordering and omitted-row counts match the
   contract; `parallelism` matches sampled and wall time.
2. Every finding has a stable code and numeric evidence with explicit units.
   Changing prose does not change the code. Threshold boundaries and suggested
   causes agree with the measurements. High classification counts never assert
   the removed whole-destination validation behavior. The entry routine never
   triggers the dominant-routine rule.
3. A worker-idle finding uses the effective pool size, including an explicitly
   limited pool, and handles an empty measurement interval without division errors.
4. Equivalent executable artifacts produce equal digests and changed executable
   content changes it. Effective defaults and explicit runtime overrides appear in
   metadata. A build without revision metadata reports `null` without failing.
5. CLI coverage confirms version 2 on success and runtime failure, unchanged
   program output and exit codes, and rejection of existing report destinations.
   Sentinel argument and environment values do not appear in the report.
6. Concurrent profiling in one process is either counted per VM or flagged, as
   chosen, and covered by a test.
7. Existing report consumers and the handbook use version 2 field names. A
   documented agent workflow locates a hot routine, reads structured evidence
   and checks comparability before evaluating a second report.
8. Run the repository checks applicable to the implementation under the
   [development process](../development-process.md#required-checks). Reuse
   existing CLI and VM coverage where it proves the contract. Confirm bounded
   report output and check that profiling-disabled VM performance stays within
   normal measurement noise using the established benchmark workflow.

For this planning-only delivery, verify relative links and `git diff --check`.
Do not run implementation tests or benchmarks merely to create the plan.
