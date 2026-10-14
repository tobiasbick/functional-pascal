# AP42.1: Collection contract

Package: [AP42](README.md). Status: complete; agreed contract recorded.

This work package records the accepted decisions. Compiler, runtime, migration,
tests, and current language-reference changes belong to
[AP42.2](02-collection-operations.md), which is implemented and independently reviewed.

## String indexing and membership

Read a string scalar only with `Text[i]`. Indexes remain zero-based Unicode
scalar positions, and invalid indexes retain the current runtime bounds error.
Remove native `CharAt`. Keep `SetCharAt` and `SetCharAtInPlace`, including their
scalar validation and AP37 value/mutation contracts. AP42 adds no writable
string indexing.

Use the existing `in` operator for membership:

| Receiver | Canonical expression | Meaning |
|----------|----------------------|---------|
| Array | `Value in Items` | An element equals Value under the existing equality/type rules. |
| String | `Sub in Text` | Text contains the substring Sub, including a single scalar. |
| Dictionary | `Key in Values` | Values contains Key; this does not search dictionary values. |
| Subrange | `Value in RangeType` | Retain the existing subrange membership contract. |

Remove native array/string `Contains` and dictionary `ContainsKey`. Preserve
existing operand compatibility, empty-substring behavior, and evaluation rules.
Membership remains usable in `requires` and `ensures` under their current
restrictions. Removing the native aliases does not introduce a purity model,
expand callable contract expressions, or reopen AP25.

## Repetition

Retain only `RepeatStr` and `RepeatStrInPlace`. Both accept arbitrary strings,
including empty, single-scalar, and multi-scalar receivers. Remove native
`FromChar` and `FromCharInPlace`, including their implicit single-scalar
validation.

Preserve the existing repetition-count contract: `N <= 0` produces the empty
string; positive N is limited to 1,000,000 repetitions. A larger positive count
retains the existing runtime error. AP42 does not add a negative-count error.

User APIs that require one scalar state that invariant explicitly, for example
`requires Text.Length() = 1;`. Migration must inspect existing `FromChar` uses:
where its validation enforces a meaningful user invariant, retain that invariant
explicitly rather than silently accepting arbitrary strings. Preserve AP37's
storage, evaluation, and failure guarantees for the InPlace operation.

## Search results and lookup

Use Option to represent absence rather than an integer sentinel:

| Native operation | Implemented result |
|------------------|--------------------|
| Array `IndexOf(Value)` | `Option of integer` |
| Array `FindIndex(Predicate)` | `Option of integer` |
| String `IndexOf(Sub)` | `Option of integer` |
| String `LastIndexOf(Sub)` | `Option of integer` |
| Array `Find(Predicate)` | `Option of T` (unchanged) |
| Dictionary `Get(Key)` | `Option of V` (unchanged) |

Index searches return `Some(Index)` when found and `None` otherwise. These
native operations no longer return `-1` for absence. Retain their names: value
search, predicate search, first/last position, and returning an element have
different purposes.

- Array indexes remain zero-based. Empty-array index searches return `None`;
  `FindIndex` does not invoke its predicate for an empty array.
- String indexes remain Unicode scalar positions. For an empty substring,
  `IndexOf` returns `Some(0)` and `LastIndexOf` returns `Some(Text.Length())`.
  Both return `Some(0)` for an empty receiver and empty substring.
- Preserve existing comparison, traversal, early-exit, argument evaluation,
  callback invocation, and callback-failure behavior.
- Dictionary `Get` and direct indexing retain their meaningful difference:
  optional lookup versus a runtime error for a missing key. Do not add `Get`
  to arrays or strings, or change their direct-index bounds errors.
- Preserve AP34 inference and AP36's complete positional-or-named argument
  lists, including the visible parameter names of retained native operations.

## Static sortability

`Sort` and `SortInPlace` sort in ascending order using the same ordering as `<`.
Sorting is stable: values that compare equal retain their input order. AP33
numeric comparisons and finite-real rules remain authoritative.

Supported element types are integer, real, boolean, string, subranges, distinct
scalar types, and enums without payload fields. Preserve aliases, nominal type
identity, and the receiver's element type in the result and updated storage.

Reject records, enums with payload fields, Option, Result, arrays, dictionaries,
callables, tasks, and channels statically. Equality support alone does not make
a type sortable. Rejection depends on the static element type, so unsupported
typed empty arrays fail just like populated arrays.

Reuse `Comparable` as the generic ordering requirement and explicitly extend
its existing admissibility to enums without payload fields. Comparisons,
`Comparable`, and native sorting must share one ordering-capability rule;
AP42 does not add a separate `Sortable` or `Orderable` constraint or a comparator
API. A generic array requires `T: Comparable` or the stronger existing
`T: Numeric` constraint. An unconstrained T or `T: Printable` alone is
insufficient.

Retain AP37 behavior: `Sort` produces a new array value; `SortInPlace` requires
writable storage and has no result. Keep its storage eligibility, single
evaluation, alias, lifetime, and failure guarantees.

## Behavior-preserving migration

Migrate resolved native calls rather than matching method names alone. Same-name
user methods and unrelated `-1` values are outside the selected removals.
Update actual consumers of search results to handle Option explicitly.

Membership replacement can change evaluation order. The old
`MakeArray().Contains(MakeValue())` evaluates the array first, while
`MakeValue() in MakeArray()` evaluates the value first. Preserve the old order
and evaluate each operand once, for example:

```fpas
const Items := MakeArray();
const Value := MakeValue();
const Found := Value in Items;
```

Temporary bindings must remain within the original branch, short-circuit path,
or loop iteration. Do not hoist work into paths where it was not evaluated.
Apply the same care to string/dictionary membership and InPlace receivers.
Contract consumers must retain their permitted expression forms and operand
behavior without adding new contract syntax or callable permissions.

## Verification contract

AP42.2 demonstrates these outcomes:

| Area | AP42.2 outcome |
|------|----------------|
| Canonical forms | Selected native aliases are rejected; indexing, `in`, and retained operations work. |
| Unicode and repetition | Scalar positions, bounds failures, count limits, and explicit user scalar invariants are preserved. |
| Search absence | All four index-search APIs return Option consistently, including empty-input edge cases. |
| Ordering | Supported types sort stably; unsupported empty/populated arrays and insufficient generic constraints fail statically. |
| Migration | Operand effects, single evaluation, short-circuiting, and callback behavior are preserved. |
| Contracts and tooling | Membership remains permitted; diagnostics, completion, generated catalogs, and cached interfaces match the new contract. |

## Replaced decisions and remaining questions

AP42 replaces AP06's selected alias entries and integer/sentinel index-search
results, removes AP37's `FromChar`/`FromCharInPlace` pair, and extends AP24's
`Comparable` admissibility to payload-free enums. Their other delivered
contracts and completed status remain intact. AP33, AP34, AP36, AP37's remaining
mutation rules, and AP41 qualification rules are preserved.

No selected AP42 language decisions remain open. Current `docs/pascal` pages
describe the delivered AP42.2 contract.
