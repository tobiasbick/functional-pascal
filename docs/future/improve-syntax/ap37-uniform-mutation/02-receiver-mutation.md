# AP37.2: Receiver mutation

Package: [AP37](README.md). Status: Complete.
Prerequisite: [AP37.1's agreed contract](01-mutation-model.md).

## Delivered behavior

Native value operations preserve their receiver. Explicit InPlace counterparts
modify writable caller storage while retaining its static type. There are nine
array, four dictionary and 19 string counterparts; the exact inventory is in
the [array](../../../pascal/language/types/array/README.md),
[dictionary](../../../pascal/language/types/dictionary-operations.md) and
[string](../../../pascal/language/types/string/README.md) handbooks.

Push returns a new appended array. Pop returns the import-free generic record
ArrayPopResult of T, with Remaining and Value. PushInPlace returns no value;
PopInPlace updates the receiver and returns the removed element. Both empty-pop
forms retain FP5003. MapInPlace/FlatMapInPlace preserve exact element/value types.
Queries and type-changing conversions have no artificial mutating counterparts.

## Implementation and migration

- The native catalog registers mutation explicitly; arbitrary user InPlace names
  do not imply writable Self or a new receiver mechanism.
- Shared var-storage checks cover readonly/computed roots, nested fields/elements,
  aliases, captures and forbidden task crossings. User Self remains read-only.
- Lowering selects receiver paths before arguments, reads storage after argument
  effects, and commits transformations only after successful computation.
  Callback snapshots and completed side effects follow the recorded contract.
- Generic result fields use erased layouts. Reference suffixes are specialized
  without losing their caller storage paths; reads/results/commits use the correct
  physical and semantic types. Nested records, arrays and forwarded var parameters
  remain writable without internal IR errors.
- Value Push/Pop are supported in go adapters; writable InPlace receivers remain
  prohibited across task boundaries.
- Existing native mutating Push/Pop consumers were migrated across source units,
  apps, examples, benchmarks, Rust/FPAS fixtures and documentation. No aliases ship.
- Editor catalog signatures, completion, recovered result fields and field hover
  reflect concrete generic arguments. The intrinsic API exporter was regenerated;
  native operations still have no public declarations in the retired helper units.
- Collection handbooks, compact reference, var/fluent/task guidance and authoring
  skill describe current behavior. No new syntax production was needed.

## Verification and independent review

- Rust formatting/build and the whole workspace test suite completed. The final
  workspace run passed with the two pre-existing network tests below excluded.
- FPAS suite: 508 passed, one skipped, zero failed (509 total).
- All 22 example/app manifests checked; release binaries and 111 standard-library
  units built. FPAS formatting and documentation links passed.
- Regression coverage includes value/original preservation, named Pop fields and
  generic constructors, all InPlace signatures in named/positional calls, same-type
  callbacks, readonly roots, go restrictions, effectful indices/arguments, callback
  failure without partial commit, generic/nested receiver paths and editor/LSP data.
- Independent review reproduced generic field and nested-path defects. All were
  repaired, covered by tracked regressions and successfully rechecked. Review
  finished without open findings, including cold/warm builds across unit boundaries.

The following untouched tests fail on both this delivery and baseline 03b98038:

- vm::hosted::net::connections::cancellable_write::tests::write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts:
  the operating system returns 32 ms for a requested 30 ms socket timeout.
- vm::hosted::net::connections::cancellable_write::tests::tcp_backpressure_can_time_out_and_cancel_without_closing:
  the socket accepts a write where the test expects cancellation under backpressure.

These are independent of AP37; network implementation and tests were not changed.
