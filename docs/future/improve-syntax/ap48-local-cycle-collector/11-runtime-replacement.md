# AP48.11: Replace runtime memory management and migrate consumers

Package: [AP48](README.md). Status: planned; blocked on prototype evaluation.
Prerequisite: [AP48.10](10-tracing-heap-prototype.md) proceed decision, with
AP48.8/AP48.9 contracts resolved. Effort: very large. Decisions: Q44 through Q48.

## Goal

Deliver one coherent production memory model. Replace the old mechanisms where
the new heap makes them redundant, migrate all consumers and remove temporary
bridges. No backward-compatible bytecode reader, Rust API adapter or dual-GC
production mode is required. Do not remove a mechanism before its safety and
semantic responsibilities have an explicit replacement.

## Integration scope

| Owner | Required migration |
| --- | --- |
| Runtime values | Complete value-kind storage/trace/drop rules, aggregate semantics, resource edges and callable code lifetime |
| Compiler and bytecode | Closure/cell instructions, register liveness/root publication, verification, serialization and invalid-artifact diagnostics |
| VM calls and callbacks | Frame roots, temporary values, nested calls, return/unwind, native re-entry and reusable sessions |
| Scheduler and tasks | Heap ownership, suspension, migration, cancellation, retries, result publication and retired-domain maintenance |
| Channels and selection | Message ownership, cancellation, abandoned cases, buffered data and resource close behavior |
| Sema | Task capabilities and diagnostics matching the selected transfer contract, without obsolete restrictions |
| Host embedding | Persistent handles, scoped access, shutdown, imports and cross-thread behavior |
| Debugger and failure reports | Rooted live inspection or detached snapshots, edits, pause/resume and retained reports |
| CLI and configuration | Collection policy for run/test/debug execution, diagnostics and removal of obsolete tuning flags |
| Repository consumers | Libraries, apps, examples, tests, generated APIs, editor tooling and benchmark generators affected by changed contracts |

Before implementation, turn this inventory into an exact file list. Runtime values
currently live under `crates/fpas-bytecode/src/value/`; execution, roots and
scheduling live under `crates/fpas-vm/src/vm/`. Host/debugger/CLI consumers span
other crates. Reorganize them if a clearer ownership boundary requires it.

## Replace responsibilities, not just types

For every removed guard, counter, classifier, lock or root registry, record the
replacement invariant and its test. Separate domain exclusion, cell mutation,
resource synchronization, executable provenance and alias semantics: one new heap
does not solve all of them automatically.

Shared storage must obey AP48.8's no-hidden-local-edge rule. Transferability checks
move to the selected boundary; avoid repeatedly traversing an unchanged graph
inside local execution. Any cached capability must remain valid under permitted
mutation, or be invalidated conservatively.

The compiler may publish live-register metadata to reduce tracing work only if
all control-flow joins, callbacks, saved states and debugger needs are covered.
Start with a correct conservative set of valid initialized values if necessary.
Do not confuse conservative retention with permission to read uninitialized slots.

## Collection policy (Q48)

Replace cell-allocation/mutation thresholds if they no longer represent heap
pressure. Candidate inputs are allocated bytes since collection, retained live
bytes and a minimum heap budget. Resolve units, overflow behavior, defaults and
propagation to all execution modes after prototype measurements.

Collection must also occur after roots disappear from retired heaps; allocation
pressure alone cannot service a heap that will never allocate again. Suspended
tasks, native waits and reusable sessions need explicit maintenance rules.
Document resource/OOM behavior and the effect of delayed external access.

Existing `--gc-cell-threshold`, `--gc-mutation-threshold` and `--gc-poll-interval`
options may be replaced or removed. Update help, configuration, constructors and
tests together. Do not preserve meaningless options or invent calibrated defaults
from noisy cloud measurements. No fixed pause or real-time guarantee is planned.

## Migration discipline

Use coherent increments that build and pass relevant checks. Temporary internal
bridges may support development but must have an explicit removal point before
AP48.12 acceptance. Keep the corrected baseline available through Git and saved
measurement artifacts; do not ship a legacy engine to preserve it.

Preserve FPAS syntax by default. Where semantics change, rewrite tests around the
selected observable behavior and explain the change. Keep memory-safety and
reclamation regressions even if their internal APIs disappear. If syntax truly
must change, update grammar, parser, formatter, diagnostics, editor support and
every repository example together. Broad user authorization permits redesign;
it does not make undocumented inconsistencies acceptable.

Update current documentation only with the delivered behavior: closure and
concurrency rules, variable references, host Rust APIs, debugger and CLI manuals,
the compact language reference, and the runtime root/execution/collection
inventories. Confirm future FPAS2 types have a documented storage/trace extension
point; do not implement unrelated new data types in this package.

## Acceptance

All value kinds and publication paths use the selected model. Host and debugger
lifetimes are covered, retired heaps reclaim, parallel tasks remain functional,
and cancellation/cleanup is deterministic where the language contract requires it.
Obsolete production GC paths, compatibility layers and misleading options are
removed. Applicable workspace, FPAS, formatting and strict Clippy checks pass.
Record actual commands and results; AP48.12 performs final stress, comparison and
package acceptance after integration is complete.
