# 05: CLI integration and report publication

Depends on [03](03-failure-capture.md) and [04](04-task-snapshots.md).
Progress lives in the [overview](README.md).

## Goal and deliverable

Implement the single `fpas run --failure-dump PATH` option and version-1 JSON
report defined by [01](01-contract.md), including new-file atomic publication
and failure reporting. A dump writer runs independently of eventual blocking
VM shutdown joins once immutable report data is ready.

## Implementation boundaries

Existing entry points are
[option parsing](../../../crates/fpas-cli/src/cli_input/options.rs),
[CLI config](../../../crates/fpas-cli/src/cli_input/types.rs),
[input resolution](../../../crates/fpas-cli/src/cli_input/mod.rs),
[help](../../../crates/fpas-cli/src/cli_input/help.rs),
[run execution](../../../crates/fpas-cli/src/cli_run.rs) and
[diagnostic output](../../../crates/fpas-cli/src/cli_output/diagnostics.rs).

Add a thematic `crates/fpas-cli/src/cli_failure_dump/` module for model
serialization and publication. Keep failure collection in the VM. Reuse existing
diagnostic codes and atomic-file facilities where they meet the no-overwrite
contract. Name exact new/moved files before implementing and split modules by
concern rather than growing `cli_run.rs` or option parsing indefinitely.

All run routes, including direct `.fpascp` and discovery, pass the same option
to execution. Without the option, no new file or capture machinery is enabled.
Keep source paths portable and do not add environment or host identity fields
to the report. FPAS value inspection itself remains read-only.

When exposing the option, add its implemented CLI/help contract and permanent
`docs/pascal/tools/failure-dump.md` schema page alongside the behavior and tests.
Step 06 checks and consolidates those docs rather than leaving a usable CLI
option undocumented between work packages.

## Acceptance criteria

- Every run input reaches the same capture/report path; other commands reject
  the option, and `--` preserves ordinary program argument forwarding.
- Success and pre-execution failures create no report or staging file.
- Escaping runtime failure writes one parseable version-1 document and retains
  existing stdout, diagnostic output and exit-code contracts.
- Existing/racing destinations are preserved, publication is atomic, and write
  failures add FP4139 without hiding the original diagnostic or leaking staging
  files. Missing parents are created only for a failure report.
- Boundary-size reports remain valid JSON and disclose every budget omission.
- Integers round-trip over their complete range; duplicate binding names and
  dictionary keys are preserved as ordered values, not collapsed JSON members.
- Metadata-free images report honest missing data rather than fabricated locals.

## Verification to record on delivery

Put wire-format and filesystem behavior at the real CLI boundary. Cover text
and JSON diagnostics, program arguments, output preservation, successful and
failing runs, missing metadata, blocked peers and publication failures.
Use existing tests for the shared CLI parser/diagnostic behavior rather than
adding duplicate tests for internal call shapes. Record commands and results.
