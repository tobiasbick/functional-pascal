# AP42: Canonical collection operations

Status: Complete; implemented and independently reviewed.
Effort: medium to large. Completion is tracked in the [central plan](../README.md).

## Original findings

- String indexing and CharAt read the same Unicode scalar.
- Membership had both `in` and Contains/ContainsKey forms.
- FromChar repeated one scalar; RepeatStr also handled this input, but did
  not provide the same single-scalar validation for arbitrary strings.
- Find returned Option while FindIndex/IndexOf used -1 for absence.
- Sort could accept an unsupported static element type during checking, then
  succeed for an empty array and fail for a populated one.

Current references: [array operations](../../../pascal/language/types/array/basics.md),
[higher-order operations](../../../pascal/language/types/array/higher-order.md),
[string editing](../../../pascal/language/types/string/edit.md),
[string searching](../../../pascal/language/types/string/search.md), and
[native calls](../../../pascal/language/functions/fluent-calls.md).

## Implemented contract

The user accepted the contract recorded in [AP42.1](01-collection-contract.md):

- Read a string scalar with `Text[i]`; remove native `CharAt`. Keep zero-based
  Unicode scalar indexing, bounds errors, and the separate `SetCharAt` and
  `SetCharAtInPlace` update operations.
- Use `in` for membership; remove native array/string `Contains` and dictionary
  `ContainsKey`. Preserve element, substring, key, and subrange membership rules,
  including membership in contracts under their existing restrictions.
- Use `RepeatStr` and `RepeatStrInPlace`; remove `FromChar` and `FromCharInPlace`.
  Preserve repetition counts and limits. User APIs needing a single scalar
  express that invariant with an explicit precondition.
- Return `Option of integer` from array `IndexOf`/`FindIndex` and string
  `IndexOf`/`LastIndexOf`. Retain element-valued `Find` and dictionary `Get`,
  with their existing Option results, and failing direct indexed lookup.
- Check `Sort` and `SortInPlace` element types statically, including empty
  arrays. Sorting is ascending and stable, using the order of `<`. Reuse
  `Comparable`, extending it to enums without payload fields, and retain
  AP33 numeric ordering and AP37 value/InPlace behavior.

No selected AP42 language decisions remain open. [AP42.2](02-collection-operations.md)
records the delivered implementation, migration, regressions, and independent
review. Current language references describe this contract.

## Dependencies and work packages

AP06 supplies the catalog. AP42 replaces its selected collection aliases and
integer/sentinel search results, removes AP37's `FromChar` pair, and extends
AP24's `Comparable` admissibility to payload-free enums. These earlier packages
retain their completed status and remaining contracts. Preserve AP33 numeric
ordering, AP34 inference, AP36 argument styles, AP37 mutation guarantees, and
AP41 qualification rules.

- [x] [AP42.1: Collection contract](01-collection-contract.md)
- [x] [AP42.2: Collection operations](02-collection-operations.md)

## Acceptance

Each selected operation has one documented form and absence/error contract.
Unsupported Sort types fail independent of runtime length; generic sorting
requires the shared ordering capability. Migration preserves evaluation order,
single-scalar invariants where required by user APIs, and contract-expression
capabilities. Compiler, runtime, consumers, tooling, generated catalogs, and
documentation agree on the recorded contract. Contextual empty arrays and
generic or chained inputs obey the same static sorting rule. AP42.2 records
validation results and the existing unrelated timing-test exceptions.
