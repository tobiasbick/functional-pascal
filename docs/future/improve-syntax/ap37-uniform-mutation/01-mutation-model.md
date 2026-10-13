# AP37.1: Mutation model

Package: [AP37](README.md). Status: Complete (decision recorded).
This contract is implemented by AP37.2; see the current
[collection handbook](../../../pascal/language/types/collection-mutation.md).

## Native collection contract

Without InPlace, an operation does not directly mutate its receiver. InPlace
requires writable storage and preserves its static type. Value transformations
support chaining. Mutating transformations normally return no value; PopInPlace
returns the removed element. Const roots and computed values are not writable.

| Operations | Agreed treatment |
| --- | --- |
| Array Sort, Reverse, Filter, Slice, Concat | Value operation plus same-type InPlace counterpart. |
| Array Push | Returns appended array; PushInPlace updates storage and returns no value. |
| Array Pop | Named result with Remaining: array of T and Value: T. |
| Array PopInPlace | Updates storage and returns removed T, without a redundant array result. |
| Dictionary Remove, Merge, Filter | Value operation plus counterpart retaining key/value types. |
| String transformations returning strings | Eligible for InPlace counterparts, including trimming, replacement, reversal, insertion, deletion, casing, padding, slicing and scalar transformations. |
| Array Map/FlatMap and dictionary Map | Value forms may change element/value type; InPlace forms retain it exactly. |
| Queries such as Length, Contains, Find, Get, Any, All | No artificial InPlace counterpart. |
| Join, Split, Keys, Values, general Reduce | No counterpart; these compute a different kind of result. |
| ForEach | No counterpart; callback effects are separate from receiver mutation. |

Both Pop forms retain the runtime error on empty arrays. Failed empty PopInPlace
leaves storage unchanged. Pop uses named fields Remaining and Value, not positional
access. Its concrete public type is the import-free generic record
ArrayPopResult of T, with Remaining: array of T and Value: T. Both empty-pop
forms retain the existing FP5003 runtime error.

```pascal
const Items := [10, 20, 30];
const Popped := Items.Pop();
const Remaining := Popped.Remaining; // [10, 20]
const Last := Popped.Value;          // 30; Items is unchanged

var Working := Items;
Working.SortInPlace();
Working.PushInPlace(40);
const Removed := Working.PopInPlace(); // 40; Working is [10, 20, 30]
```

## Storage and user-defined operations

User-defined mutation uses ordinary explicit var arguments. Record Self remains
read-only; bound methods capture values. Callable signatures may retain explicit
var parameters, but writable references cannot escape or cross go boundaries.
Native InPlace operations reuse current writable-root, record-field and array-
element rules; dictionary entries and other rejected var targets remain rejected.

Preserve exact typing, aliasing, lifetimes, failure protections and once-only
written-order evaluation of receiver selection and arguments. AP36's complete
positional-or-named lists remain, including Name := var Storage for explicit var
arguments. No new receiver grammar is required.

InPlace preserves value semantics: modifying one collection variable must not
alter another value copy. It is not an allocation-free guarantee. Callback effects
are separate; do not describe all value operations as pure.

## Superseded decisions

Replace AP06/AP17's unmarked native Push/Pop mutation with mandatory
PushInPlace/PopInPlace names. Introduce value Push/Pop under the original names
and migrate all existing mutating consumers semantically. Keep Sort and Reverse
without Sorted/Reversed aliases. Both Items := Items.Sort() and
Items.SortInPlace() are deliberately allowed.

The decision gate is closed. AP37.2 delivered this contract; no general
suffix-based mutation mechanism for user routines or shared-state handles is added.
