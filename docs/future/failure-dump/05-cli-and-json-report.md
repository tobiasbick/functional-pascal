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

## Delivered implementation

The single option reaches source, project, workspace, direct-image and discovery
runs through the same execution path. Path validation is read-only; capture and
the writer are absent without the option. An independent writer receives immutable
VM observations, plans borrowed JSON views within the escaped-byte bound, and
publishes through a synced sibling staging file and atomic no-overwrite hard link.
Unsupported publication adds FP4139 after the original diagnostic. Ordinary
ownership cleanup still precedes final error selection where required.

The serializer preserves integer decimal strings, duplicate lexical binding
names and dictionary key/value pairs. Cause/main identities and attached
diagnostics precede optional values. Known source coordinates survive omitted
path names. Additional byte clipping retains valid JSON and explicit omission
markers; minimal fallbacks also preserve diagnostic-text omission reasons.

The implemented schema and limits live in the permanent
[failure-report page](../../pascal/tools/failure-dump.md), linked from the CLI,
tools index, diagnostics and compact authoring reference. The root README,
AGENTS instructions and FPAS authoring skill remain accurate: language rules,
test/format commands and authoring requirements have not changed. Step 06 owns
the remaining full-feature acceptance/performance work and documentation
consolidation before removing this temporary directory.

## Verification evidence

- Real CLI tests cover every input route, arguments after `--`, text/JSON
  diagnostics, handled failures, success and pre-execution failures, existing
  destinations, parent creation, publication failure, concurrent writers and
  preservation of a destination created after validation. An observer parses
  files as they become visible, checking complete atomic publication.
- Value tests cover nested frames, parameters/locals/globals, integer extremes,
  Unicode, duplicate names with distinct scopes and ordered dictionary keys.
  Direct images without debug bindings retain stored positions and honestly
  report missing locals.
- Escaped-value tests produce valid reports above 6 MiB but within the 8 MiB
  ceiling. Two independent review findings were reproduced before repair:
  large primary values omitted an attached child diagnostic, and a source-name
  budget overflow erased a known source ID/position. Both CLI regressions pass
  after repairs at the planner/source owners. Review also checked minimal
  fallback omissions and schema documentation; no findings remain open.
- A process-isolated real native operation proves that reporting preserves
  required owned cleanup. Its bounded helper is ignored during direct discovery
  and explicitly executed by its passing parent. Earlier VM process tests own
  the distinct guarantee of notification before remaining shutdown joins.
- The focused CLI suite passes 13 tests. Its directly ignored native helper is
  executed by the parent above. The documented JSON parses with `Std.Json` and
  matches an actual generated report; the handbook example passes the FPAS
  documentation checker and its extracted source passes `fpas fmt --check`.
- Formatting and build pass. Workspace tests pass, including 586 CLI unit tests,
  647 VM unit tests, FPAS suites, handbook examples, debugger protocols, other
  packages and doctests. The only exclusions are the same two unchanged TCP tests
  documented in [step 02](02-shared-inspection.md#verification-evidence):
  `tcp_backpressure_can_time_out_and_cancel_without_closing` and
  `write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts`.
  After the final minimal-fallback omission repair, the build, 13 CLI boundary
  tests and existing compact-reference test were rerun for the final code.
  All 147 relative file links in the changed Markdown resolve, and `git diff --check`
  passes. Step 06 remains open; this step does not claim complete D1 performance verification.

Validation commands (from the repository root):

```text
cargo fmt --all -- --check
cargo build
cargo test --workspace -- --skip tcp_backpressure_can_time_out_and_cancel_without_closing --skip write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts
cargo test -p fpas-cli --test failure_dump
cargo test -p fpas-cli agent_reference_examples_check_and_run
bin/fpas run tools/doc-examples/doc-examples.fpasprj -- --document docs/pascal/tools/failure-dump.md
```
