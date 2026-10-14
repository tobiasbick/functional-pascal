# AP45.1: Specify and verify snapshot iteration

Package: [AP45](README.md). Status: complete; implemented, verified, and independently reviewed.

The permission to mutate the iterated source in this delivered contract is
superseded by the agreed [AP45.2 read-only iteration revision](02-read-only-iteration.md).
The implementation below records the original delivery; AP45.2 owns the revision.

## Implemented contract

The iterable expression runs once per loop entry, including empty collections.
Arrays retain their original element values and order; dictionaries retain their
original keys in insertion order. Source mutation or reassignment leaves that
sequence unchanged. Separate dictionary lookups read the current dictionary and
retain ordinary missing-key failures. Nested loop entries capture new snapshots.

Loop bindings remain read-only and scoped to the loop. Empty collections,
`break` and `continue` retain their ordinary behavior. Channel handles and closure
environments remain shared through value copies; iteration adds no deep cloning
or concurrency guarantee. No new syntax, live-iteration API or iterator framework
is introduced.

## Implementation ownership

- `crates/fpas-compiler/src/lowering/control_flow.rs` retains its existing
  `lower_for_in` implementation: evaluate once, obtain dictionary keys once,
  store the collection and length, then read each element from that collection.
- `crates/fpas-bytecode/src/value/array.rs` and
  `crates/fpas-bytecode/src/value/aggregate.rs` retain copy-on-write collection
  storage. Mutating the source preserves the loop's
  captured values without cloning shared resources.
- `crates/fpas-compiler/src/lowering/closures/discover.rs` visits ordinary
  discarded expressions without registering synthetic task adapters. This
  repairs discarded `PopInPlace` calls needed by the removal regression;
  `Expr::Go` still registers its own task adapter. Ordinary discard semantics
  and the existing worker/capture restrictions are preserved.
- `docs/pascal/language/control-flow/for-in.md`,
  `docs/pascal/language/types/collection-mutation.md` and
  `docs/pascal/agent-reference.md` specify the same contract with executable examples.
  The grammar and repository application sources require no migration.

## Regression ownership

- `tests/stdlib/language/for_in_snapshot_test.fpas` owns positive end-to-end
  coverage: array append/removal/replacement/reassignment; dictionary
  insertion/deletion/reinsertion/reassignment and current-value lookups;
  once-per-entry evaluation including empty collections; nested snapshots and
  innermost `break`/`continue`; shared closure environments and channel handles.
  Bounded fixtures and visit guards prevent live-iteration regressions from
  hanging. The existing suite glob includes the file.
- `crates/fpas-compiler/src/tests/control_flow/iteration.rs` owns FP5019 when
  a snapshotted key has been removed from the current dictionary before lookup.
- `crates/fpas-compiler/src/tests/discard.rs` protects discarded InPlace removal
  through local and record-field storage. Existing discard and concurrency tests
  retain coverage of ordinary results, task starts and capture restrictions.
- Existing semantic tests in `crates/fpas-sema/src/tests/stmt/for_loops.rs`
  protect immutable bindings and lexical scope for arrays and dictionaries.
- The existing CLI compact-reference test checks and executes the updated
  `ReferenceControl` program, including mutation during snapshot iteration.

## Verification and open follow-ups

Rust formatting, workspace build and workspace tests pass with the two known
baseline TCP cases excluded. FPAS regressions, source formatting, app/example
project checks, handbook examples, compact-reference execution, documentation
links and release distribution checks pass. Independent review found no
blocking issues.

The excluded tests are
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` and
`tcp_backpressure_can_time_out_and_cancel_without_closing`. Their implementations
and tests are unchanged by AP45; baseline evidence is retained in
[AP41's verification notes](../ap41-canonical-qualified-names/02-qualified-name-rules.md).
No selected AP45 work remains open.
