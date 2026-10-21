# AP48.7: Automatic scheduling and acceptance

Package: [AP48](README.md). Status: complete. Effort: medium to large.
Prerequisite: [AP48.6](06-cycle-collector.md) complete.

## Delivered contract

VM, callback and debugger domains collect local cycles automatically. Each domain
counts allocation/mutation pressure and completed execution batches, requesting a
whole-domain pass at the first reached threshold. Existing exclusion and complete
root publication determine when service is safe. Collection introduces no global
execution lock, collector thread or FPAS source syntax.

Q42 defines the heuristic initial defaults and positive CLI overrides:

| Trigger | Default | CLI override |
| --- | --- | --- |
| Newly allocated cells | 4096 | `--gc-cell-threshold=N` |
| Successful cell mutations | 8192 | `--gc-mutation-threshold=N` |
| Completed VM execution batches | 256 | `--gc-poll-interval=N` |

Periodic polls cover garbage created by dropping values without new allocations.
Each completed dispatch batch contributes once, including nested callbacks; retries
at the same safe point do not advance the batch counter. Pressure resets only after
successful service. Busy/incomplete passes retain pending work through the existing
request-generation and permit-release handshake.

Root removal, quiescent edge-removing mutation, retirement and publication before
parking request collection independently of running thresholds. The requesting
thread services a quiescent domain synchronously once complete roots and exclusion
permit it. Saved tasks and published native waits can collect without dispatching
FPAS; opaque native calls defer until return/unwind. A full trace has no fixed
maximum pause guarantee.

Reference counting still releases acyclic data immediately. Reachable values retain
identity, aliases, copy-on-write and callable provenance. Transfers, task/resource
ownership, cancellation and cleanup rules remain unchanged. Raw administrative cell
handles and snapshots require the established guarded publication/root discipline.
No parameter can bypass complete-root publication, domain exclusion or interrupted
commit poisoning.

## Configuration and propagation

Run/test/debugger commands accept both `--option=N` and `--option N`. Values must be
positive decimal integers representable by the shared runtime configuration; zero,
negative, malformed, overflowing, missing and duplicate values are argument errors.
Omitted options use defaults. Options after `--` remain program arguments. Existing
invocations remain valid. Check/build/format commands reject these execution options.

One effective configuration flows through source/project/workspace and verified-image
runs, normal/parallel/isolated tests and setup/teardown hooks, and batch/JSONL/DAP
sessions. Task domains, retries, reusable callbacks, detached inspections and debugger
helper construction inherit the owning runtime's settings. Existing embedding
constructor signatures remain; additive `with_collection_schedule` methods configure
VMs, callback sessions and debugger targets/sessions before execution.

The internal explicit-maintenance route and diagnostic policies remain available.
There is no public FPAS GC unit, manual collection command or `--gc=off` option.

## Implementation ownership

- `crates/fpas-bytecode/src/value/roots/scheduling.rs` owns the nonzero shared
  configuration/defaults, per-domain pressure and mandatory boundary notifications.
  `roots/{access,maintenance,execution,mod}.rs` owns policy, successful reset and
  request/exclusion coordination. `value/{domain,cell}.rs` records successful events.
- `crates/fpas-vm/src/vm/domain/collector.rs` installs scheduled collection and
  constructs independent domains with inherited settings. `dispatch.rs` records
  completed batches before deciding whether nested caller roots must be published;
  `tasks/state.rs` requests a pass after complete saved-state publication.
- VM/callback/debugger configuration methods, task/group spawn, worker templates and
  debugger detachment/enum construction propagate settings without merging domain
  identities. `fpas-debug/src/target.rs` applies configuration at protocol launch.
- `crates/fpas-cli/src/cli_input/collection.rs` owns shared parsing/validation.
  Input types/help, `cli_run.rs`, `cli_debug/mod.rs` and `cli_test/` owners carry
  effective configuration through execution, hooks and the isolated worker wire.
- The closure/runtime ownership and CLI/test/debugger handbook pages plus the domain
  inventories document automatic collection, options, units and raw access rules.
  The compact language reference is confirmed; no syntax or generated Std API changes.

See the [collector inventory](../../../../crates/fpas-vm/src/vm/domain/COLLECTION.md),
[execution protocol](../../../../crates/fpas-vm/src/vm/domain/EXECUTION.md) and
[root inventory](../../../../crates/fpas-vm/src/vm/domain/ROOTS.md).

## Acceptance and verification

- Independent allocation, successful-mutation and periodic triggers use actual
  reclamation; periodic service works without new allocations. Busy preparation
  preserves cells and pending work for retry; successful passes reset pressure.
- Real dispatch repeatedly creates/drops self-recursive mutable closures under both
  defaults and smaller schedules. Live cells remain bounded while retained closure
  aliases preserve state; cycles reclaim during execution and after retirement.
- Independent roots survive non-final release and quiescent mutation after task/VM
  destruction. Release during collection triggers another pass without lost work;
  an unrelated VM progresses while collection is blocked in its own domain.
- Saved completion/continuation state, native callers and published waits retain
  reachable cycles. Opaque execution defers service; owner/attempt cases close before
  retry and inspection roots remain independent. Callback/inspection churn releases
  cycles after each final export/handle without closing the reusable session.
- Task/pool/helper configuration and distinct ownership identities are preserved.
  CLI tests cover defaults/individual/combined overrides, invalid values, program
  argument separation and exact nonzero serialization through worker requests.
  Real processes exercise source/project/cached/image runs, isolated tests and
  batch/JSONL/DAP launch with aggressive collection settings.
- Existing deep/shared graph, ownership/provenance, migration/channel/cancellation/
  retry, artifact/cache, embedding and debugger regressions run with automatic
  factories. Raw-handle test observations explicitly register Host roots; callback
  and mutation assertions retain their original meaning. The TUI background-failure
  fixture waits for failure delivery with a monotonic deadline instead of assuming
  that its earlier ready signal guarantees task completion.

Applicable Rust build/workspace tests, strict Clippy, Rust/Pascal formatting,
standalone Pascal suite, affected projects and documentation checks are recorded in
the delivery report. Independent GPT-6.1 Sol review and repair review are complete.

## Package completion and later tuning

Q43 approves activation and completion after correctness acceptance, required
checks, repaired review and documentation. The defaults are heuristic policy
choices, not measured optima. Controlled throughput/memory/pause budgets are not
activation or package-completion prerequisites.

AP48.1 through AP48.7 and package acceptance are complete. Later controlled local
measurements may compare the original baseline, ownership enforcement alone and
automatic collection. Throughput, live allocations, pause distributions and root
release latency can guide independent threshold tuning. Cloud runs provide
correctness evidence only; experiment output belongs under `.temp-data/`.
