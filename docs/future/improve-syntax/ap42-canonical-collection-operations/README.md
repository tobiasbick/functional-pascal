# AP42: Canonical collection operations

Status: Decision pending. Effort: medium to large. Completion is tracked in
the [central plan](../README.md).

## Findings

- String indexing and CharAt read the same Unicode scalar.
- Membership has both `in` and Contains/ContainsKey forms.
- FromChar repeats one scalar; RepeatStr also handles this input, but does
  not provide the same single-scalar validation for arbitrary strings.
- Find returns Option while FindIndex/IndexOf use -1 for absence.
- Sort can accept an unsupported static element type during checking, then
  succeed for an empty array and fail for a populated one.

Evidence: [array operations](../../../pascal/language/types/array/basics.md),
[higher-order operations](../../../pascal/language/types/array/higher-order.md),
[string editing](../../../pascal/language/types/string/edit.md),
[string searching](../../../pascal/language/types/string/search.md), and
[native calls](../../../pascal/language/functions/fluent-calls.md).

## Proposed direction and open decisions

Prefer indexing, membership via `in`, one repetition operation, Option for
search absence, and static rejection of unsupported sorting. Decide which
aliases to remove, whether single-scalar validation remains its own operation,
and which orderable element types Sort supports. Preserve Option-returning
Get versus failing indexed access as a meaningful distinction.

## Dependencies and work packages

AP06 supplies the catalog. AP42.1 precedes AP42.2. AP37's mutation decision
must be respected; AP42 does not change receiver mutation. Coordinate numeric
ordering with AP33 only if the selected Sort contract requires its changes.

- [ ] [AP42.1: Collection contract](01-collection-contract.md)
- [ ] [AP42.2: Collection operations](02-collection-operations.md)

## Acceptance

Each selected operation has one documented form and absence/error contract.
Unsupported Sort types fail independent of runtime length. Migration preserves
argument evaluation order and contract-expression capabilities.
