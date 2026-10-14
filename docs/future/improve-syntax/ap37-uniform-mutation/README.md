# AP37: Uniform mutation visibility

Status: Complete. Effort: large.
Completion is tracked in the [central plan](../README.md).
AP37.1 and AP37.2 are complete, including independent review and migrations.

## Agreed contract

Native collection operations without InPlace do not directly mutate their
receiver. Mutating counterparts require the InPlace suffix, writable storage,
and an unchanged receiver type. Apply this rule consistently to eligible array,
dictionary and string transformations. Keep Sort/SortInPlace and
Reverse/ReverseInPlace, without Sorted/Reversed or additional free/dot aliases.

Push returns a new appended array. PushInPlace appends to caller storage and
returns no value. Pop returns a named result with Remaining (the shortened new
array) and Value (the removed last element); the original array is unchanged.
PopInPlace shortens caller storage and returns only the removed element.
Both Pop forms retain the existing runtime error on empty arrays.

User-defined mutation retains explicit var parameters and call-site markers.
Record Self remains read-only; bound methods capture values. Preserve exact var
types, writable-root/alias/lifetime checks, single evaluation in written order,
failure protections, go restrictions and AP36's complete positional-or-named lists.
No mutating user Self, bound writable receiver or new receiver grammar is added.

InPlace preserves collection value semantics; it does not promise zero allocation.
Callbacks may have separate side effects, so value operations are not necessarily
pure. Both assignment of a transformed value and InPlace calls are intentionally
available. The earlier recommendation against paired APIs is superseded.

## Dependencies and migration

AP06/AP17/AP36 remain completed. AP37 explicitly replaces AP06/AP17's unmarked
native mutating Push/Pop exception. Existing mutating calls were migrated to
PushInPlace/PopInPlace; their behavior was preserved. Push and Pop now have the
agreed value-returning contracts.
AP40 retains shared-state/task ownership.
[AP42](../ap42-canonical-collection-operations/README.md) removes the delivered
FromChar/FromCharInPlace pair in favor of RepeatStr/RepeatStrInPlace, changes
index-search absence to Option, and enforces static stable sorting. AP37's other
value/InPlace contracts, Pop empty-array error, and completed status remain intact.

- [x] [AP37.1: Mutation model](01-mutation-model.md)
- [x] [AP37.2: Receiver mutation](02-receiver-mutation.md)

## Acceptance

The compiler, native catalog, runtime, formatter, editor, API exporter and handbook
implement the agreed contracts. Storage and value-semantics protections remain
intact. There are 32 InPlace counterparts: nine array, four dictionary and
19 string operations. Queries and type-changing conversions remain value-only.

Verification: 508 FPAS tests passed, one skipped; all 22 example/app manifests
checked; release binaries and 111 standard-library units built. The independent
review found generic field/reference type-erasure defects, all repaired and
rechecked with tracked regressions, including nested and imported result types.
The Rust workspace passes with two unchanged network tests excluded; both failures
were reproduced on baseline commit 03b98038. Details are in AP37.2.
