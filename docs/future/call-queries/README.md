# Call query implementation plan

Status: planned. Deliver `query callees` first, then `query callers`. Both are
tooling additions over saved, fully checked sources; no FPAS syntax changes are
selected. They complete the two remaining operations in the
[source-query inventory](../cli-gap.md#source-queries-and-refactorings).

- [Callees](callees.md): operations owned by one selected source body.
- [Callers](callers.md): incoming calls and callable references to one checked
  target within the explicit project/solution context.

## Analysis constraints

Reuse [workspace analysis](../../architecture/workspaces.md), detached
[semantic snapshots](../../architecture/compiler.md#semantic-snapshots), exact
declaration/body ranges and verified FPAS DLL/source identities. Preparation,
complete-context analysis, input verification, cancellation, configuration and
process ownership already belong to these modules.

The present occurrence index records a routine call and a method-to-delegate
conversion as symbol reads. It does not publish call sites, their source-body
owners or dispatch classification. Build these facts from binding, before
closure/async lowering; do not reconstruct them from refs, names or emitted CIL.

## Owning modules

| Module | Responsibility |
| --- | --- |
| Fpas.Compiler | Record source bodies, checked operations, selected targets/signatures, dispatch and source provenance. |
| Fpas.Workspaces | Resolve selectors and canonical identities, index outgoing/incoming sites, bound results and describe search/resolution scope. |
| Fpas.Cli | Bind typed requests, use the shared analysis lifecycle, and expose JSON results/help. |

Keep one body/site model and one identity mapping for both directions. Callees
filters by owner; callers indexes those same facts by target. Do not build a
second resolver, project loader, method-name matcher or generic graph framework.
Keep files focused in the existing Analysis, Binding.Semantics and Queries topics.

## Body and site facts

Publish immutable body descriptors with snapshot-local ID, kind, exact source
regions, optional source declaration symbol and optional enclosing body ID.
Cover named/nested routines, block/expression lambdas, program entry, global
initializers and field defaults. Anonymous bodies have honest kinds/locations;
do not invent source declarations or expose generated CLR method names.

Each site belongs to exactly one body. Binding establishes ownership; enclosing
text ranges alone cannot assign nested operations correctly. A routine owns its
requires/ensures operations, tagged by phase. A nested routine/lambda owns its
own operations, even when its declaration occurs inside another body. An
initializer owns its operations rather than attributing them to every consumer
or constructor. Preserve creation/reference sites in the enclosing body.

Each site records its exact operation extent, navigation/target anchor when
present, kind, phase, static target or explicit unknown reason, dispatch and
checked signature/type arguments. Several operations can share a source extent;
distinguish them by operation kind/accessor and a stable snapshot-local identity.
Keep original UTF-16 coordinates/newlines and copied documents. Return no bound
objects, syntax trees, live metadata readers or persistent handles.

Use exact member identities for overloads/constructors/accessors. Preserve the
logical source declaration as navigation context separately from the invoked
CLR operation. Generic incoming matching uses the checked generic definition;
each returned site retains its actual instantiation/substitutions.

## Operation policy

| Source operation | Planned fact |
| --- | --- |
| Direct routine/method call | `invoke`, selected overload and checked signature. |
| Explicit construction / source union case construction | `construct`, exact selected CLR operation where available and logical source type/case. |
| CLR property/indexer get/set, event add/remove, supported imported operator | Accessor/operator operation with exact identity; preserve multiple operations for read-modify-write. |
| Named function/method or lambda converted/passed as a callable value | `callable_reference`, separate from an invocation. |
| Delegate invocation | `delegate_invoke`, checked callable signature and receiver provenance; runtime function target unknown. |
| Virtual/interface/constrained invocation | Bound static contract plus dispatch classification; runtime implementation unknown. |
| Source storage, type tests/tokens and ordinary value reads | No invocation; ordinary refs retain their existing meaning. |
| Compiler representation, closure/state-machine, contract-failure and copy/domain helpers | Excluded generated infrastructure, never fabricated source calls. |
| Implicit using/foreach/await runtime protocols | Explicitly excluded from this first source-operation scope, with source-site category counts. |

Audit every bound expression/statement form against this policy. Imported FPAS
representations must receive the same logical classification as checked source
under the [CLR mapping](../../specs/clr-mapping.md); crossing a verified DLL must
not turn a source storage read into an extra generated-helper call. Attribute
metadata construction is compile-time data, not an executable body invocation.

Complete analysis means all inputs in the selected context were checked. It does
not prove runtime reachability, call order/counts, delegate flow or virtual target
sets. Publish the indexed/excluded operation scope and unresolved-target counts.
Intentional exclusions are scope facts; output paging sets truncation separately.
Never present an excluded or unavailable body as a complete zero-call answer.

## Delivery order

### CQ1: Shared body/operation catalogue

- [ ] Record bodies/regions and operation facts during syntax/binding, including
  overload retargeting, contract phases and initializer ownership.
- [ ] Define a complete bound-form policy; test generated/synthetic exclusions
  and source-versus-imported representation consistency.
- [ ] Add detached snapshot facts without changing existing query identities,
  expression policies, metadata exports or emitted program behavior.

### CQ2: Outgoing query

- [ ] Implement [callees](callees.md), its exact selector rules, availability,
  deterministic paging and unresolved/excluded summaries.
- [ ] Accept the packaged command with a matching SDK on both platforms.

### CQ3: Incoming query

- [ ] Implement [callers](callers.md) as an index over the same sites, using
  existing canonical declaration/export identities across checked projects.
- [ ] Keep actual calls, callable references and unattributable dynamic sites
  distinct; accept both platforms independently.

### CQ4: Documentation and final acceptance

- [ ] Update the implemented CLI/agent references, compiler/workspace architecture,
  [coverage map](../../../tests/coverage.md) and CLI/editor gap inventories.
- [ ] Finish the matrix below, retain concise evidence in the test documentation
  and remove completed steps/plans. Do not advertise planned commands as available.

## Acceptance

| Check | Native Windows | Linux/WSL |
| --- | --- | --- |
| Body/site semantics and existing definition/type/refs/find/inspect regression | [ ] | [ ] |
| Callees: selection, ownership, exact targets, availability and bounds | [ ] | [ ] |
| Callers: project scope, DLL mapping, callable uses and dispatch uncertainty | [ ] | [ ] |
| Fresh packaged SDK consumer and published launcher queries after dist | [ ] | [ ] |
| Changed inputs, no execution, cancellation and process cleanup | [ ] | [ ] |
| Full Debug suite, Release build and focused Release/SDK/documentation tests | [ ] | [ ] |

Follow the [development workflow](../../build/development.md), with default SDK
parallelism. Extend existing compiler analysis and workspace/CLI owners; prove
compiler facts there and use integration tests for transport, SDK and lifecycle
risks. Keep one primary regression per contract instead of duplicating fixtures.
Record platform-only or unavailable-backend skips explicitly. One platform cannot
complete the other column. Preserve the existing real-debugger regression.

LSP, editable/invalid buffers, whole-program reachability, interprocedural delegate
analysis, recursive graph traversal, refactorings, new language features and query
MCP tools remain separate work.
