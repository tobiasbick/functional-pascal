# AP44.1: Enforce unambiguous enum values

Package: [AP44](README.md). Status: planned; not implemented.

## Scope

Implement the package's declaration restrictions without changing enum runtime
representation or adding a second enum identity model.

## Implementation and migration

1. Check resolved backing values in `crates/fpas-sema/src/check/decl/types/enums.rs`.
   Report explicit/explicit and explicit/implicit collisions with both variant
   names, the conflicting value, and a concrete repair. Detect collisions in
   either declaration order; do not merely compare adjacent values.
2. Reject any explicit backing value in an enum containing payload fields,
   regardless of the order of those variants. Keep fieldless alternatives valid.
3. Inspect enum parsing, constant evaluation, case coverage, imported type
   interfaces and editor checking. Reuse declaration checks rather than adding
   different local/imported rules. Ensure pre-change compiled units cannot
   bypass validation; invalidate incompatible sidecars if required.
4. Inventory repository consumers, generated declarations and Rust fixtures.
   Remove unused backing annotations; choose distinct values where variants
   really are distinct. If code relies on aliases, inspect its intended behavior
   and migrate explicitly rather than blindly renumbering it.
5. Update `docs/pascal/language/types/enums.md`, the grammar's semantic notes,
   affected diagnostics and the compact reference. Confirm sorting, equality,
   pattern coverage and authoring guidance describe the same identity rule.

## Verification

- Reject duplicate explicit values, nonadjacent collisions, implicit successors
  colliding with earlier explicit values, and explicit values colliding with
  earlier implicit values. Cover negative values and integer boundaries.
- Reject mixed backing/payload declarations in both orders, including generic
  enums. Accept ordinary payload enums with fieldless alternatives.
- Preserve distinct explicit numbering, restarted sequences, backing-value
  ordering, stable sorting, exhaustive cases and existing overflow diagnostics.
- Check independently compiled/imported units and stale sidecar handling.
- Run the relevant sema/compiler/editor tests, affected FPAS consumers and the
  [required checks](../development-process.md#required-checks). Review the final
  change before marking AP44 complete.
