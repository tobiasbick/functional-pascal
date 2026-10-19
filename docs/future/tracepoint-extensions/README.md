# D5: Tracepoint extensions

Status: Packages 1–3, 5 and 6 completed and independently reviewed.
Whole aggregates remain deferred. This is the temporary D5 extension index, not an AP package or
an implemented language specification.

The scalar-path implementation scope is complete: local record fields, fixed
array positions and activation-scoped capture bindings. Permanent docs/tests
own the completed packages, so their temporary implementation files are removed.
Stable anonymous names are also complete. Concrete debug projections improve
type metadata without adding source-language features.

Current behavior belongs to the permanent
[tracepoint contract](../../pascal/tools/debugger-tracepoints.md). D1 failure
reports, D3 batch inspection and D4 seeded scheduling are already implemented.
D2 record/replay is outside this plan. JSONL removal remains deferred.

## Order and status

| Order | Package | Status | Dependency |
| --- | --- | --- | --- |
| 1 | [Scalar fields of local records](../../pascal/tools/debugger-tracepoints.md#selecting-storage) | Completed; independently reviewed | Existing scalar local/parameter tracing |
| 2 | [Fixed array positions](../../pascal/tools/debugger-tracepoints.md#selecting-storage) | Completed; independently reviewed | Package 1 and shared metadata path traversal |
| 3 | [Scalar capture bindings](../../pascal/tools/debugger-tracepoints.md#observed-changes) | Completed; independently reviewed | Packages 1–2 and exact capture provenance |
| 4 | [Stable anonymous callback names](../../pascal/tools/debugger-tracepoints.md#anonymous-callback-names) | Completed; independently reviewed | Exact routine selectors and capture provenance |
| 5 | [Concrete debug projections](../../pascal/tools/debugger-tracepoints.md#stored-fields-and-array-positions) | Completed; independently reviewed | Stable callback names; D5-M1/M2 together |
| — | [Whole aggregate history](04-aggregate-decision.md) | Deferred; no implementation scheduled | Concrete dogfooding evidence |

Use these status words consistently: **Planned**, **In progress**, **In review**,
**Completed**, **Decision open**, **Deferred**. Deferred work has no implied
implementation commitment. An implementation blocker is recorded separately
with its concrete cause and next action.

No further debugger implementation package is scheduled. Continue dogfooding the
delivered scalar paths. The first Notes case found and fixed an application selection bug,
covered by [headless regressions](../../../tests/apps/notes/note_search_selection_test.fpas)
and [save-failure checks](../../../tests/apps/notes/note_search_dirty_test.fpas).
The Notes application fix was delivered separately from the debugger packages.
Reopen the [aggregate assessment](04-aggregate-decision.md) only when concrete
cases demonstrate missing information that selected paths or D1/D3 cannot provide.
Continuous history of dormant escaped cells is also deferred and unscheduled.

## Verification and review

Packages 1–3 received independent subagent review; no findings remain open.
Review fixes include exact enum IDs, capture definition scope/declaration order,
strict capture ABI representation and the compiler scope correction for forwarded
zero-word reads. Regression cases include real callback tail-frame reuse,
shared/independent cells, escaped/dormant captures, alias ordering, exact event
capacities, absent indexes, parameter snapshots and UTF-8 bounds.

Checks for packages 1–3 passed:

- 312 compiler tests; 754 VM tests (two baseline ignored, two known cloud TCP
  timing tests excluded); 303 debugger tests; 85 real-process batch tests.
- Compact agent-reference examples and all five tracepoint documentation examples.
- Strict workspace Clippy for all targets, Rust formatting, changed Markdown links
  and `git diff --check`. Cloud timings are not performance acceptance data.

Package 6 addresses the D5-M1/M2 follow-ups: erased generic record fields and
nominal array element names. Its permanent contract is documented under
[stored fields and array positions](../../pascal/tools/debugger-tracepoints.md#stored-fields-and-array-positions).

### Stable anonymous names (package 5)

Published `$closure_N` names are allocated once per literal in the lowering
source walk, with separate ordinals per executable lexical owner. Semantic AST
identity keys stay private; private generic scope names use source positions.
Root literals use the reserved `$entry_programname` owner to avoid collisions
with legal same-named source routines. Named helpers keep the full owner chain.
Bound-method, parser and intrinsic/native task adapters were audited: their
existing names use deterministic function IDs, not AST addresses, and do not
consume anonymous source ordinals. Their provenance classification stays intact.

Two independent subagent reviews are complete. The found root/routine naming
collision was fixed and rechecked, including mixed-case headings and distinct
capture sources. Regression coverage includes retained defaults and aliases,
hoisting versus lexical source order, nested owners, sibling generic helpers,
independent/shared captures, fresh-process exact selector reuse, unit cache
reuse and object/image round trips.

Validation passed: 315 compiler tests, 802 Sema tests, 89 real-process batch
tests, 451 VM debugger tests and 27 debugger report tests. The four new CLI
tests passed again after adding explicit unchanged-sidecar checks. All 13
compact-reference examples and six tracepoint examples passed, including the
new anonymous counter. Strict workspace Clippy, Rust formatting and diff checks
passed. Cloud timings are not performance acceptance data.

Permanent docs/tests now own this package; its temporary plan file is removed.

### Concrete debug projections (package 6)

Exact globals and source bindings retain bounded, acyclic source-type projections
beside the erased operational layouts. Applied generic fields and nested nominal
array element names now survive lowering, objects, linking and portable images.
References and captures use their exact declaration provenance; unresolved generic
types remain unsupported. Compiler cutoffs resume existing concrete operational
metadata rather than disabling previously supported scalar selectors. Bounds are
depth 32 (root 0), 4,096 nodes and 65,536 nominal-name UTF-8 bytes per root.
Object/program formats advance to 14/22; instruction format 23 and runtime
storage layouts stay unchanged. Old artifacts require rebuilding.

Independent review findings were fixed and rechecked: recursive expansion reserves
later scalar siblings; same-span pattern bindings retain separate checked types;
synthetic lowering temporaries do not inherit source projections; imported erased
terminals and array/reference containers reconcile with canonical linked layouts
without changing eligibility or hiding known type contradictions. Unsupported
children remain unsupported after linking. The strict final verifier is retained.

Regression coverage includes generic instantiations sharing one runtime layout,
nested and recursive records, aliases, distinct/subrange/enum array elements,
mutable aliases, replacements, absent indexes, immutable/mutable/transitive
captures, mixed pattern captures, expansion cutoffs, trace limits, atomic setup
failures and source/project/cached-object/image equivalence. Permanent docs/tests
own this package; its temporary implementation file is removed. The D5-M1/M2
intake entries are resolved. Whole aggregates and dormant escaped-cell history
remain deferred.

Validation passed: 317 compiler tests, 803 Sema tests, 43 IR validation tests,
the bytecode/object/program/linker suites, 458 VM debugger tests, 303 debugger
tests and 101 real-process batch tests. All 13 Notes tests, all 13 compact-reference
examples and seven tracepoint examples passed. Strict workspace Clippy for all
targets, Rust formatting, changed Markdown links/anchors and diff checks passed.
Cloud runs provide correctness evidence, not performance acceptance measurements.

## Shared decisions

The unchanged program/object format rule below describes packages 1–3 and 5.
Package 6 adds debug metadata and versions the affected encodings separately.

- Keep one batch command, `--trace`, and one result document. No new session
  protocol, alias flag, source instrumentation or interactive setup is needed.
- Keep the existing roots: full executable global name, or exact
  `Routine::Binding`. Extend the path after that root with stored fields and,
  in package 2, literal array positions. Match identifier names ASCII
  case-insensitively and deduplicate by resolved metadata identity.
- Resolve the complete request atomically before statements and initializers.
  Reject ambiguous names, unsupported steps, getters, calls and expressions.
  Never execute user code to resolve a selector or inspect its value.
- Ordinary locals remain scoped to their declaration/call. Capture selectors
  in package 3 describe the selected callback's own active capture binding;
  they do not silently extend a declaring local's history after scope exit.
- Follow the selected storage path, not a previously observed record object,
  array element value or host pointer. Attribute each successful commit to its
  actual writer; attribute delayed results to their original call instruction.
- Preserve initialization, consecutive equality, request order within one
  commit, invocation identities and explicit incomplete-history semantics.
  Failed writes and unrelated sibling changes produce no false scalar events.
- Store detached scalar comparison values only. Do not clone or retain entire
  aggregate roots for comparison. Temporary source access/alias leases must
  end with their owning frame or synchronous callback.
- Preserve existing ceilings: 64 selectors, 4,096 selector bytes, 1,000 default
  events/10,000 maximum, 65,536 bytes per scalar string, 16 MiB string-comparison
  work, 2 MiB escaped trace JSON and 8 MiB total report JSON. Keep the existing
  8 MiB global-commit and 8 MiB local-comparison string budgets.
- Keep at most 1,024 live selected path instances, counting each live
  `(watch_id, instance_id)` pair even when paths share a declaration lifetime.
  A declaration's selected paths share its `instance_id`; each call has its
  existing `activation_id`. Renew the instance when a declaration executes
  again. Document this counting rule in the owning contract and effective limits.
- Bounds are charged before retaining or traversing data. Unknown equality or
  an unreadable value ends coverage explicitly; presentation clipping is never
  used as the equality test. No sampling, ring buffer or lossy digest shortcut.
- Reuse current debug metadata and keep program/object formats unchanged.
  Additive identity/result fields are permitted if documented and tested;
  retain the wire shapes of already supported selectors. If essential metadata
  is missing, record the gap and its smallest compiler-side remedy first.
- Keep untraced execution, DAP/JSONL, ordinary batch inspection and D4 scheduling
  behavior intact. Do not make globals shared between worker tasks.

## Delivery and verification

For every implemented package:

1. Mark it In progress; record implementation decisions and newly found gaps.
2. Implement focused binder/runtime changes and meaningful source-to-VM tests.
3. Add real-process CLI tests, including source/project/portable-image targets,
   atomic setup failures, bounded reports, breakpoints and runtime failures.
4. Exercise recursion, reused callbacks, worker tasks, seeded execution and
   delayed callback/suspension results at the end of lexical scopes.
5. Run affected tests, strict workspace Clippy, formatting and diff checks.
   Run broader regressions when the changed paths justify them. Cloud runs
   provide correctness evidence, not valid performance measurements.
6. Mark In review, request an independent subagent review, fix findings and
   rerun affected checks. A finding is complete only after its fix is checked.
7. Update the permanent tracepoint page, batch/CLI documentation, compact
   reference and authoring skill where relevant. Check changed examples/links.
8. Update the wishlist and this index. Remove a completed package file once
   permanent docs/tests own its scope, replacing its index link with that
   permanent contract. Remove this directory when no open package remains.

Do not describe planned selector forms as already implemented in the handbook.
Commit and push are separate user actions, not completion requirements here.
