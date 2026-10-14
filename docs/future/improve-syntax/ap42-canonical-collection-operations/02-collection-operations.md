# AP42.2: Collection operations

Package: [AP42](README.md). Status: complete; implemented and independently reviewed.

Prerequisite: [AP42.1](01-collection-contract.md) is complete. Its recorded
contract is delivered; no selected AP42 language decisions remain open.

## Scope and affected areas

Delivered these changes together:

1. Remove native string `CharAt`, array/string `Contains`, dictionary
   `ContainsKey`, and string `FromChar`/`FromCharInPlace`. Retain `Text[i]`,
   `in`, `SetCharAt`/`SetCharAtInPlace`, and `RepeatStr`/`RepeatStrInPlace`
   with the precise contracts recorded in AP42.1.
2. Change array `IndexOf`/`FindIndex` and string `IndexOf`/`LastIndexOf` to
   `Option of integer`, including all found, absent, and empty-input paths.
   Retain `Find`, dictionary `Get`, and direct indexed lookup contracts.
3. Enforce ascending, stable `Sort`/`SortInPlace` through a shared static
   ordering-capability rule. Extend `Comparable` to payload-free enums and
   align generic checks, comparisons, and sorting. Unsupported typed empty
   arrays must fail checking; `Numeric` implies sufficient ordering capability,
   while unconstrained or Printable-only type parameters do not.

Inspect native signatures under `crates/fpas-sema/src/std_registry/builtins/`,
type/constraint predicates, contract validation, lowering and runtime native
dispatch, generated API declarations, diagnostics, and language-service
completion. Reuse existing membership and ordering logic; remove obsolete
native paths and catalog entries rather than leaving compatibility aliases.

Preserve AP33 numeric ordering and AP34 inference, AP36 positional-or-fully-named
arguments, AP37 value/InPlace storage and failure guarantees, and AP41 qualified
names. Apply the search signatures and ordering constraints to imported routines
and generic instantiations as well as local code. Update compiled-unit/native
artifact compatibility as needed so old integer-returning searches or unchecked
sorting cannot reappear through reused artifacts. Let the build system rebuild
derived artifacts; do not hand-edit them.

## Migration and documentation

Use resolved operations to migrate native calls, preserving same-name user
methods. Replace search-derived integer/sentinel handling with explicit Option
handling; do not rewrite unrelated `-1` values. Keep retained native parameter
names and AP36 argument-list styles.

Membership migration must preserve receiver-before-argument effects and single
evaluation, using temporary bindings where necessary. Keep evaluation within
the original branch, short-circuit path, and loop iteration. Check contract
consumers separately: membership remains expressible through `in` under the
existing restrictions without new callable permissions or an AP25 purity model.

Inspect `FromChar` uses for meaningful single-scalar invariants. Where required,
make validation explicit in the user API, such as a `requires Text.Length() = 1;`
precondition, before using `RepeatStr`. Preserve existing repetition behavior
for zero/negative counts and the positive-count limit of 1,000,000.

Update native array/string/dictionary references, operators, generic constraints,
contracts, compact reference, generated API catalogs, diagnostics/editor
guidance, authoring guidance, examples, tests, Std units, and application
consumers. Adjust [shared catalog checks](../shared-constraints.md) to match
the delivered operation set; do not preserve an obsolete fixed entry count.
Document Option-valued search, Unicode/empty-input rules, explicit scalar
invariants, supported sort types, generic constraints, and stable ordering.

At delivery, mark the replaced portions of AP06's catalog/search contract,
AP24's `Comparable` rules, and AP37's `FromChar` pair with links to AP42.
Preserve those packages' completed status and remaining historical scope.

## Verification

Cover the recorded contract with positive and negative regressions:

- Accepted indexing, membership, repetition, search, and update forms; rejected
  removed native aliases; retained same-name user methods.
- Unicode scalar indexing/search, including supplementary scalars and combining
  sequences; existing bounds errors and `SetCharAt` validation.
- Empty/single/multiple-scalar repetition, zero and negative counts, the accepted
  positive boundary, over-limit failure, and preserved user scalar invariants.
- All four Option index-search signatures with found/missing values, empty arrays,
  absent/empty substrings, and empty strings. Verify predicate call count/order,
  early exit, failures, and imported result typing. Confirm unchanged `Find`,
  dictionary `Get`, and direct-index failure behavior.
- Stable ascending sorting for each supported type, including payload-free enums,
  subranges, distinct scalars, aliases, and equal real values such as signed zero.
  Reject each unsupported element category for typed empty and populated arrays.
  Verify `Comparable` and `Numeric` generics, rejected unconstrained/Printable-only
  generics, and consistent `<`/constraint/native-sort admissibility.
- AP37 copying versus writable InPlace storage, receiver evaluation, alias,
  lifetime, and failure behavior; no new result from `SortInPlace`.
- Membership operand effects, single evaluation, short-circuiting, branch locality,
  per-iteration evaluation, and membership in `requires`/`ensures` under the
  existing restrictions.
- Cold and reused compiled-unit builds, generic imported consumers, generated
  declarations, native completion, and actionable diagnostics for removed APIs,
  Option-result misuse, and unsupported sort types.

Run appropriate Rust compiler/runtime/workspace checks, the FPAS suite, affected
Std/application/distribution checks, formatter/editor checks, and documentation
link/example checks. Have an independent subagent review the completed work,
resolve findings, and then update AP42.2 and the central delivery status.

## Implementation and review

- The shared native catalog removes the selected aliases and declares all four
  index searches as `Option of integer`. Semantic validation, direct intrinsics,
  synchronous and suspended callbacks, diagnostics, and native editor signatures
  use those results. Removed spellings suggest the canonical indexing, membership,
  or repetition form; integer-sentinel comparisons explain Some/None handling.
- `Ty::is_comparable` supplies the shared comparison, generic-constraint, and
  native-sort capability, including payload-free enums. A shared scalar comparator
  supplies runtime ordering; stable sorting preserves equal values and signed
  zeros, validates finite real values, and retains nominal element types.
- Contextual receiver inference reuses catalog argument/result evidence and the
  existing deferred-inference checks. Unsupported empty literal inputs fail even
  through aliases, typed returns/arguments, wrappers, and type-changing or terminal
  native chains. Comparable/Numeric generic and supported scalar inference remain
  valid; missing type evidence cannot bypass the ordering requirement.
- Migration preserves same-name record methods, local operand evaluation, search
  callback behavior, and AP37 value/InPlace guarantees. Existing FromChar consumers
  used literal single scalars; explicit user preconditions have separate success
  and failure coverage. String-index lowering now borrows original index AST nodes,
  retaining operand type metadata for negative and computed indexes.
- Compiled-unit and program envelopes are version 19; bytecode is version 22.
  Old collection artifacts are rebuilt by the normal build path. Cold and reused
  unit tests cover imported Option results and constrained enum sorting; independent
  review also verified rejection of an AP41-built integer-search artifact.
- Array/string/dictionary references, constraints, contracts, the compact reference,
  authoring guidance, Std consumers, applications, examples, and tests are migrated.
  AP06/AP24/AP37 identify their replaced portions while retaining completed status.
  Generated API checks and the shared catalog check use the current operation set.

Independent subagent review approved the delivery with no remaining actionable
findings. It verified the contextual empty-sort repair and improved sentinel
diagnostics, and exercised distinct scalars, signed-zero stability, repetition
limits, contract membership, effectful InPlace receivers, single evaluation,
nested search callbacks, and cold/reused imported enum aliases.

## Validation results

Validation on 2026-10-14:

- Rust formatting and workspace build pass. Workspace tests cover 770 semantic
  and 300 compiler unit tests, imported/cached collection contracts, runtime paths,
  and language-service/editor protocol suites. The confirming run excludes only
  the two pre-existing TCP exceptions recorded below. A sporadic debugger assertion
  in `instruction_and_timeout_limits_are_reported` occurred during parallel load;
  the complete 12-test `fpas-debug` jsonl suite passes on isolated recheck.
- The FPAS suite passes 515 tests with one intentional skip, including four new
  canonical-collection programs. All 120 application/example project and standalone
  checks pass; source formatting is clean.
- Release binaries and all 111 bundled standard-library units build. Generated
  Std-unit API export passes; native signatures come from the shared catalog.
- Editor TypeScript compilation, grammar checks, and contract checks pass.
  Documentation links resolve: 1,982 relative file links in 317 documents, with
  no missing files or heading candidates. Compact-reference examples are checked
  by the CLI documentation suite.

The pre-existing TCP exceptions are
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` and
`tcp_backpressure_can_time_out_and_cancel_without_closing`, already documented in
[AP41 validation](../ap41-canonical-qualified-names/02-qualified-name-rules.md#review-and-validation).
The relevant networking implementation/tests and debugger timing assertion are
unchanged by AP42. They do not leave an AP42 contract or language decision open.
