# D5.6: Concrete debug projections and nominal element names

Status: Planned; follows [stable callback names](05-stable-callback-names.md).
This package jointly addresses
[D5-M1](../compiler-panic-followups.md#d5-m1-erased-generic-record-fields) and
[D5-M2](../compiler-panic-followups.md#d5-m2-erased-nominal-array-element-names).

## Purpose and boundaries

Retain the concrete checked type of a selected stored path independently of its
erased operational record/array layout. `Box of integer.Item` should bind as an
integer path; elements declared as `array of Dollars` should retain `Dollars`
where the nominal type exists in source semantics.

- Keep current source syntax, runtime layouts and type erasure. Add debug-only
  structural/projection metadata, rooted in exact source bindings and globals.
- Separate machine-readable applied types and nominal names from formatted
  container display strings. Never parse display strings or infer eligibility
  by running an initializer or observing a runtime value.
- Preserve source declaration identity through generics, references and capture
  provenance. Different instantiations sharing one runtime record layout must
  retain different concrete debug projections where their source types differ.
- Continue supporting scalar leaves only. Recursive metadata must be bounded;
  unsupported or genuinely unknown paths still reject atomically before execution.
- Preserve existing selector spelling, event semantics, path/instance accounting,
  detached scalar retention and explicit unknown-coverage behavior.
- Carry metadata through lowering, objects, linking and portable images. If
  encoding changes, explicitly version and document those formats; the original
  packages' unchanged-format constraint does not apply to this separate delivery.

## Work and acceptance

- [ ] Specify the smallest bounded projection graph and its validation rules,
  distinguishing applied semantic types, operational layout slots and display names.
- [ ] Retain applied generic fields and nested nominal array element names at
  each supported global/local/parameter/capture root.
- [ ] Encode, validate, link and round-trip the metadata with explicit format
  versions where needed; reject malformed, cyclic/unbounded or mismatched paths.
- [ ] Integrate exact pre-execution trace binding without changing program storage.
- [ ] Test multiple instantiations, nested/recursive generic records, arrays,
  aliases, distinct containers, nominal scalar/subrange/enum leaves and references.
- [ ] Verify mutable aliases, record/array replacement, known absent indexes,
  captures, limits and source/project/object/image equivalence.
- [ ] Verify old concretely typed selectors retain their identities and behavior;
  unsupported values do not gain misleading scalar histories.
- [ ] Run relevant regressions and strict checks; obtain independent review,
  fix findings and update docs, intake entries and wishlist.

Whole aggregates, dormant escaped-cell tracking, new debug modes and D2 replay
remain outside scope. Actual debugging cases should drive any additional work.
