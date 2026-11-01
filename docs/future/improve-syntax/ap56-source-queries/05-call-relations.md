# AP56.5: Add direct call relationships

Package: [AP56](README.md). Status: complete; depends on AP56.2.
Uses Q85 and Q88. AP56.3 supplies name/reference selectors when available;
positional call queries must remain usable independently of those selectors.

## Contract and scope

Implement the detail plan's [direct call queries](../../agent-source-queries.md#phase-3-direct-callers-and-callees).
Commands:

```text
fpas query callers --symbol App.Math.Clamp --context examples/pascal/units-basic/units-basic.fpasprj
fpas query callees examples/pascal/units-basic/src/math_utils.fpas:5:17 --context examples/pascal/units-basic/units-basic.fpasprj
```

Derive relationships from snapshot-owned checked allocations: the original AST
and retained expressions in `record_defaults`/`static_bound_declarations`. Keep
identity-keyed metadata attached to its actual allocation. Reuse binding references,
method/fluent/intrinsic targets, constructor classifications and named-argument
ordering in `AnalysisMetadata`. None alone proves a complete source call graph.
Record any missing source declaration link at its semantic resolution owner;
never match a target by display text, arity or debugger function ID.

Keep direct calls, callable-value uses, callback arguments and unresolved dynamic
calls in distinct collections. Include exact call spans, containing callable,
target identity when proven, invocation kind and argument/parameter mapping.
Preserve each call site and generic source declaration identity. Attribute nested
routine/closure bodies to that routine/closure, not its enclosing callable.
Use explicit owners for program entry and unit initialization.
Record-field defaults have their own declaration/default owner and physical
source span. They execute in the declaration environment when construction needs
that default, not at unit initialization. Inspect retained static-bound expressions
under their declaration owner too. Deduplicate original/retained representations
by source span and semantic context; do not duplicate a default call at every
construction or generic instantiation. Missing checked facts remain unavailable.

Enumerate expression calls, procedure statements, postfix/method calls and
`go` calls. Distinguish record/enum construction, distinct conversions and native
operations from source routines. A callable variable/field/result remains dynamic
without compiler proof of its exact target. A callback passage is a use, not
evidence that the receiving routine invokes it.

`callees` returns direct relationships inside the selected body. `callers`
reverses proven edges in the eligible analyzed context and includes unresolved
sites that cannot be excluded as uses of the selected routine. An initial
scope-wide unresolved inventory is acceptable; narrowing needs compiler proof.
Reuse AP56.2's operation-independent scope, cancellation and final verification.
Do not require a daemon, persisted index or VM execution.

Report `direct_edges_complete` separately from `runtime_targets_complete`.
Relevant unresolved targets, omitted bodies and failed analysis remain explicit;
zero direct edges does not establish zero possible executions. Apply the shared
1,000-default/10,000-maximum limit independently to each relation collection,
with bounded traversal, stable ordering and known/unknown totals.

## Source-form audit

Before adding metadata, map each source form to existing checked facts and its
exact owning resolution path. Classify it as proven, explicitly unresolved or
unavailable. Keep this matrix in this work file with the implemented coverage.
Include retained record defaults and static-bound expressions explicitly; an
original-AST-only walk does not prove completeness for those forms. State which
compiler-generated invocations, such as resource cleanup, are outside the source
call inventory; `runtime_targets_complete` concerns the inventoried sites, not
every runtime effect. A parameter change must still account for all declaration
uses, including compiler-bound uses outside ordinary call syntax.
This is agent-owned engineering work, not a user approval gate.

### Resolution-owner audit

The collector now lives in `navigation/source_query/calls/` in the language
service: `resolution.rs` classifies compiler evidence, `walk/` visits original and
retained allocations, `index.rs` links exact declarations, and `projection.rs`
serves both CLI and rich LSP. `hierarchy.rs` supplies snapshot-bound anonymous and
named owners for standard LSP hierarchy. The following matrix records the
semantic evidence used by those implementations.

| Source form | Existing checked evidence and owner | Collector treatment / missing fact |
| --- | --- | --- |
| Named expression or statement call, including imports and recursion | `check/expr/calls/mod.rs::resolve_call_target`, `check/stmt/calls.rs::check_call_stmt`; `record_initialization_reference` retains the selected kind, canonical import identity and local declaration span | `AnalysisMetadata.source_bindings` now exports this evidence, including qualified and nonlocal bindings. Its regression distinguishes imported/local routines from callable values. A callable type alone does not prove a routine target. |
| Callable variable, parameter, field or indexed value | Selected symbol kind, `member_value_calls`, expression/path types; `check/expr/calls/indexed.rs` | Explicit dynamic site; do not infer the initializer's target. |
| Instance/static record call and bound method value | `method_calls` and `bound_methods`, populated by `check/expr/calls/methods/` and `check/expr/bound_method.rs` | Map the compiler-selected nominal owner/member to its source declaration; preserve receiver and written argument spans. |
| Native intrinsic and fluent operation | `intrinsic_calls`, `fluent_calls` and `native_values`, owned by the corresponding call/designator checkers | Known compiler identity and builtin origin; no fabricated source implementation. |
| Record/enum construction and scalar conversion | `record_constructions`, exported `enum_constructions`, `distinct_conversions`, `subrange_conversions` at the call checker | Exclude constructor/conversion syntax from source routine edges; traverse its argument expressions. |
| Named actual arguments | `named_argument_orders` from `check/calls/named.rs` | Preserve written evaluation order and report its checked parameter mapping. |
| Callable value / callback argument | Value designator resolution, bound/native value metadata; receiving call signature and named-argument mapping | Separate value/callback use; passage does not establish execution. |
| Nested routine, anonymous closure and task spawn | Snapshot routine/closure AST, `closure_infos`, `nested_routine_captures`, and the task-call checker | Separate physical containing owner; tag proven spawn edges without attributing nested bodies to outer routines. |
| Record-field default | `record_defaults` retains `Arc<Expr>` allocations checked in `check/decl/types/records/defaults.rs::check_record_defaults` | Walk the retained allocation once under its declaration/default owner; the original AST clone has different identity-keyed metadata. |
| Static-bound constant/subrange expression | `static_bound_declarations` retains checked declaration allocations | Traverse these allocations under their original source owner and deduplicate exact source sites. |
| Resource release and other implicit invocation | `resource_releases` records canonical release identity/signature/native state | Explicit source-inventory exclusion; retain it as a blocker for changing an affected callable's signature. |

Paths in this table are relative to `crates/fpas-sema/src/`.

AP56.7 consumes the full typed relation inventory, not clipped CLI JSON.
It must see callable-value/callback uses and unresolved sites as well as direct
calls. An unavailable mapping blocks the later edit instead of guessing.

## Required LSP connection

Implement standard prepare/incoming/outgoing Call Hierarchy using the shared
relation inventory and snapshot-bound targets. Preserve exact call sites when
grouping relationships for LSP. Advertise the capability only with real handlers.
Also expose callers/callees through `fpas/sourceQuery`, retaining unresolved
sites, related uses and both completeness fields. Standard Call Hierarchy is a
projection of proven direct edges, not proof of all runtime targets. Check all
relevant editor overlays and reject stale hierarchy data. Follow the
[adapter contract](adapter-contract.md#required-operation-mapping).

## Affected files

| Action | Repository path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-language-service/src/navigation/source_query/calls.rs` | Snapshot-bound call inventory, query projection and coverage |
| Modify | `crates/fpas-language-service/src/navigation/source_query/` (`mod.rs`, `model.rs`, `scope.rs`) | Shared results, scope and cancellation |
| Modify when a missing link is demonstrated | `crates/fpas-sema/src/check/context.rs`, `crates/fpas-sema/src/check/expr/calls/` (`mod.rs`, `methods/mod.rs`, `fluent.rs`), `crates/fpas-sema/src/check/stmt/calls.rs`, `crates/fpas-sema/src/check/calls/named.rs` | Retain exact checked targets/argument mappings at their existing owner |
| Reuse | `crates/fpas-parser/src/ast/` (`expr.rs`, `stmt.rs`, `routines.rs`) | Existing source forms and spans; no new grammar |
| Modify | `crates/fpas-cli/src/cli_input/query.rs`, `crates/fpas-cli/src/cli_query/` (`worker.rs`, `report.rs`) | Call-query options, typed rendering and focused help |
| Create/register | `crates/fpas-language-service/tests/source_query_calls.rs`, `crates/fpas-language-service/tests/suites/navigation_edits.rs` | Proven identities, containing owners and completeness |
| Create/register | `crates/fpas-cli/tests/source_queries/calls.rs`, `crates/fpas-cli/tests/source_queries/mod.rs` | Public collection limits, outcomes and selector behavior |
| Create/modify | LSP Call Hierarchy and extension registration paths in `adapter-contract.md`, including `crates/fpas-lsp/src/call_hierarchy.rs`, `src/server/call_hierarchy.rs`, `src/server/backend.rs` | Standard and rich call-query adapters on the same collector |
| Create/register | `crates/fpas-lsp/tests/call_hierarchy.rs`, `tests/suites/workspace.rs`; extend `tests/source_tools.rs` under that crate | Actual hierarchy handlers/capabilities, exact spans, unresolved metadata and dirty/stale inputs |

Keep the relation collector reusable by refactoring without routing through CLI
serialization. Split call traversal from projection if a file becomes too large.

## Documentation and verification

Update focused `callers`/`callees` help, the source-query handbook, CLI reference
and agent tools reference. State scope and uncertainty in the actual results.

Cover direct/mutual recursion, same-name declarations, aliases, methods, generic
calls, named arguments, nested routines/closures, spawns and initializers. Include
record defaults with several constructions, shadowed names at construction,
retained static-bound expressions and unsupported implicit invocation uses.
callable fields/results, callbacks, native calls and constructors. Test complete
empty results separately from unresolved, excluded and broken-source cases.
Reordered discovery must preserve output; small limits must retain uncertainty.
Unchanged references work, while changed source or scope invalidates results.

Reuse navigation/semantic fixtures where they already prove a contract. Register
new cases in the existing `navigation_edits` and CLI `tools` targets. Compile
then execute those targets as in [AP56.2](02-foundation.md), including focused
help checks. Apply [shared constraints](../shared-constraints.md), validate
documentation links and run `git diff --check`.
Run the adapter contract's LSP workspace/protocol tests. Normalize grouped call
sites and compare them with CLI/core direct edges on equal saved inputs; verify
that rich LSP responses preserve uncertainty. Include dirty call sites, stale
hierarchy items, URI/UTF-16 conversion and cancellation. CLI-only delivery is incomplete.
