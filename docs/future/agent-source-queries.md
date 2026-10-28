# Source queries for coding agents

> **Status: proposed specification; implementation not started.** The repository
> behavior described below was checked on `refactor/runtime-architecture` at
> `3f8a22390`. Commands marked proposed are not available yet. This document
> defines a later implementation, without changing FPAS syntax or execution.

## Problem and goal

Agents can already ask semantic questions at a saved source position. Finding
that position, collecting a useful implementation context, and separating calls
from other references still require several operations and manual interpretation.
The goal is a bounded, reproducible source interface that makes those steps
explicit and eventually supports checked edits.

FPAS source remains the authoritative program representation. Symbols, types,
relations, and edit plans are derived from exact source snapshots and compiler
analysis. They do not constitute a new source format or a primary graph store.

Priority is: name discovery and selection, focused inspection, then direct call
relationships. Adding a parameter and updating known calls comes after the
read-only tools. Existing rename provides useful infrastructure, not proof that
arbitrary refactorings are safe.

## Verified current behavior

The implemented user contracts are documented in [source queries](../pascal/tools/source-queries.md),
[editor integration](../pascal/tools/editor-integration.md), and
[diagnostics](../pascal/tools/diagnostics.md). The following distinguishes those
contracts from the additions in this proposal.

| Area | Present behavior and source evidence |
| --- | --- |
| CLI selection | [`cli_input/query.rs`](../../crates/fpas-cli/src/cli_input/query.rs) accepts `refs`, `definition`, `type`, and `rename`. All require one `file.fpas:line:column`; there is no name selector, `find`, `inspect`, `callers`, or `callees`. |
| Read-only queries | [`worker.rs`](../../crates/fpas-cli/src/cli_query/worker.rs) calls the shared [`LanguageService::source_query`](../../crates/fpas-language-service/src/navigation/source_query.rs). It analyzes saved sources without bytecode compilation or program execution. `type` uses checked AST metadata, not hover text. |
| Read contract | [`report.rs`](../../crates/fpas-cli/src/cli_query/report.rs) emits schema version 1 with snake_case fields, outcomes, diagnostics, locations, limits, and coverage. Even invalid requests receive JSON; a final stdout failure is exceptional. |
| Context and snapshots | [`SourceInputs`](../../crates/fpas-project/src/source_inputs.rs) freezes source and manifest bytes, bounds discovery, records canonical and lexical path identities and file selections, and verifies them before reporting. The worker exports SHA-256 hashes of exact input bytes. |
| Name search | [`workspace_symbols.rs`](../../crates/fpas-language-service/src/navigation/workspace_symbols.rs) searches indexed names case-insensitively, ranks exact short names before partial matches, sorts deterministically, and truncates at 100. Its vector result does not report truncation or query coverage. [`WorkspaceSymbolIndex`](../../crates/fpas-language-service/src/symbols/index.rs) retains ambiguous qualified and unqualified candidates. |
| Navigation metadata | [`DocumentSymbol`](../../crates/fpas-language-service/src/symbols/document.rs) has qualified names, kinds, declaration/name/scope spans, visibility, detail, and optional callable signatures. [`navigation/service.rs`](../../crates/fpas-language-service/src/navigation/service.rs) shares resolution, references, rename, and declaration documentation. |
| Analysis reuse | [`DocumentSnapshot`](../../crates/fpas-language-service/src/document/snapshot.rs) owns immutable text, parsed AST, line index, and store-local revisions. [`AnalysisCache`](../../crates/fpas-language-service/src/analysis/cache.rs) keys completed analyses by participating revisions and project context; request forks share it within a running service. Separate CLI worker processes do not share that cache. |
| Rename | [`rename.rs`](../../crates/fpas-cli/src/cli_query/rename.rs) pins known source snapshots, checks analysis before and after in-memory edits, checks conflicts through the service, and supports `--dry-run`. [`publication.rs`](../../crates/fpas-cli/src/cli_query/publication.rs) stages replacements and verifies source bytes before publishing. |
| Unit dependencies | [`UnitGraph`](../../crates/fpas-project/src/unit_graph/model.rs) retains parsed units, origins, direct `uses`, and source identities. Import reachability is not a function-call graph. |
| Debugger effects | [`fpas-code/src/debug/effects.rs`](../../crates/fpas-code/src/debug/effects.rs) classifies verified bytecode, follows direct call/tail-call edges to transitive effects, and marks dynamic calls and callback intrinsics conservatively. This file moved from `fpas-bytecode`; it is not a general source-call query. |

Read queries accept `--context` with a project or workspace manifest. Position
queries otherwise discover source ownership; loose files have explicitly limited
scope. Workspace queries include member projects and dependencies, with each
consumer's imports and library exports governing resolution. Consumers outside
that context are not discovered. Unused standard-library implementation bodies
are not searched for uses of a user declaration.

Current bounds are 30 seconds per invocation, 4,096 canonical input files,
8 MiB per input, 64 MiB total input, and 8 MiB escaped JSON including its newline.
`refs --limit` defaults to 1,000 and permits at most 10,000 retained uses. Preview
lines retain 160 Unicode scalars; selected textual report fields retain 4,096.
Missing semantic analysis or omitted uses produces incomplete coverage, rather
than a complete empty result. The read parent requests cancellation and allows
at most 250 ms before terminating and reaping the isolated worker tree, subject
to the documented OS and final-output limitations.

Rename differs materially: `--project` instead of `--context`, a separate
`schemaVersion`/`query`/`position` success object, and stderr diagnostics with exit
1 on failure. It has no read-query timeout or result-limit options. Its CLI
publication plan verifies pinned source bytes, but does not use the read worker's
full manifest/membership snapshot contract. Do not describe these interfaces as
already unified.

Rename stages every replacement before committing any, preserves permissions
and untouched UTF-8 bytes, and commits individual files through `AtomicWriteFile`.
It rechecks inputs and checks each edited file again before replacement. On an
ordinary publication failure it attempts restoration in reverse order; it retains
newer concurrent text instead of overwriting it and reports failed restorations.
There is no multi-file filesystem transaction, lock against other writers, or
crash-atomic guarantee. A failed final stdout write can occur after edits were
applied. Rename currently reports those problems on stderr, not in success JSON.

## Common query contract

This section specifies the intended contract. Phase 0 establishes it before the
new read operations. Compatibility with both existing schema-1 shapes is not a
requirement; update documentation, fixtures, and repository consumers together.

### Selectors and context

Proposed examples, assuming a workspace containing unit `Dungeon.Client.Requests`
and its declaration `Queue`:

```sh
fpas query find Queue --context dungeon.fpasworkspace
fpas query refs --symbol Dungeon.Client.Requests.Queue --context dungeon.fpasworkspace
fpas query definition --symbol Dungeon.Client.Requests.Queue --context dungeon.fpasworkspace
fpas query type src/client.fpas:12:5 --context dungeon.fpasworkspace
fpas query inspect --symbol Dungeon.Client.Requests.Queue --context dungeon.fpasworkspace
fpas query callers --symbol Dungeon.Client.Requests.Queue --context dungeon.fpasworkspace
fpas query callees --symbol Dungeon.Client.Requests.Queue --context dungeon.fpasworkspace
fpas query rename src/client.fpas:12:5 Submit --context dungeon.fpasworkspace --dry-run
```

`find` takes search text. Other operations accept exactly one of a position,
`--symbol <name>`, or `--target-ref <json-file>`. Reject mixed selectors, missing
values, repeated options, and unsupported operation options through the common
error envelope. Rename also requires its replacement name. Keep positional
selection for expressions, locals, aliases, and shadowed declarations.

Use `--context` consistently; remove rename's separate `--project` spelling.
Require an explicit `.fpasprj` or `.fpasworkspace` for `find` and `--symbol` in
the first version: there is no source path from which to infer ownership.
Position selection retains current discovery. A target-reference file carries
its context; an explicit context must agree with it. Directories are not context
manifests. `--std-lib` retains its meaning and participates in snapshot identity.

Symbol selection is exact, case-insensitive compiler-name matching, not fuzzy
search. Accept exact short or owner-qualified names, preserving every candidate.
Neither a short name nor a qualified string alone guarantees uniqueness across
projects or nested scopes. Resolve only after the eligible scope is fully
checked. More than one candidate yields `ambiguous_target`, candidate summaries,
and no selected target; a clipped candidate list must never become a selection.
File-local import aliases require a position or target reference, not a globally
interpreted alias string. Original unit-qualified declaration names are tooling
selectors; they do not change FPAS import syntax or visibility.

For `type --symbol`, query the declaration's checked value type; a routine value
has its callable type. It does not mean the result of some unspecified call.
Position selection keeps existing call-site and expression semantics. A unit,
type declaration, or alias with no value type returns an explicit no-value-type
result. `inspect` handles declaration information for these cases.

### Versioned result and failure model

Use a common `schema_version: 2` envelope for all queries, including rename.
Keep one UTF-8 JSON object plus newline on stdout for success and failure;
reserve stderr for inability to deliver the final report. Help remains help.
The exact operation payloads must have schema fixtures before implementation.

| Field | Proposed meaning |
| --- | --- |
| `schema_version`, `operation`, `outcome`, `exit_code` | Version, recognized operation or null, result class, matching process exit. |
| `request` | Tagged selector or search text, operation arguments, and effective options; unavailable parsed fields are null. |
| `context` | Kind, manifest, sorted projects, standard-library selection, path base, and searched scope. |
| `snapshot` | Content-derived request identity, analysis-contract identity, and hashed input inventory; null if unavailable. |
| `target`, `candidates` | Selected declaration and reusable reference, or bounded ambiguity candidates. |
| `result` | Tagged operation payload: symbols, locations, type, inspection sections, relations, or edit/publication information. |
| `diagnostics` | Original attributed FP diagnostics and structured query diagnostics with stable codes. |
| `coverage` | Analysis completeness, reasons, searched/excluded scope, returned/known-total counts per collection, and text omissions. |
| `limits` | Effective deadline, analysis/input ceilings, and all output budgets. |

This intentionally replaces read schema-1 top-level `matches`/`type`, reshapes
`request`, and replaces rename's camelCase envelope. Do not silently label these
changes version 1. Preserve source-attributed diagnostic codes, messages, hints,
and spans through the existing diagnostic renderer; never parse human messages.

| Outcome | Exit | Meaning |
| --- | --- | --- |
| `ok` | 0 | Complete answer within the stated operation and scope; empty `find` or zero uses of a resolved symbol is valid. |
| `not_found` | 1 | Complete eligible scope but no selected target or no construct supporting the requested question. |
| `incomplete` | 3 | Analysis errors, unavailable metadata, cancellation, limits, source changes, or unresolved relevant call targets; retain proven partial data. |
| `error` | 1 | Invalid input/context, read failure, ambiguous target, stale reference, conflict, or publication failure. |
| `error` | 3 | Internal or worker failure preventing a reliable answer. |

No runtime-error exit 2 applies. Complete ambiguity is an error; incomplete
discovery cannot establish uniqueness or absence and stays incomplete. A malformed
reference is invalid input; a valid but mismatching reference is `stale_target`.
Unknown totals remain null. Read results with `source_changed` are observational
only and must not authorize a follow-up edit. Text clipping and deliberately
limited context are reported separately from missing semantic analysis.

Retain one-based Unicode-scalar positions and half-open ranges with zero-based
UTF-8 byte spans. Supplementary characters, tabs, combining characters, LF, CRLF,
bare CR, and BOM must use the existing snapshot conversion rules. LSP adapters
continue to convert to UTF-16 separately. Use `/` in display paths, relative to
the reported invocation directory when possible; never confuse display paths
with physical source identities.

### Snapshot-bound target references

A reusable reference is a versioned JSON object returned as `target_ref`, not an
eternal node ID. The proposed file form avoids shell escaping and token-length
limits. Save exactly that object as `queue.target.json`, then use:

```sh
fpas query inspect --target-ref queue.target.json
```

Separate three concepts:

1. **Symbol identity:** the resolved declaration in an owning semantic context.
   The reference records owner/project, kind, qualified spelling, canonical
   defining file, and defining identifier span to disambiguate the declaration.
2. **Source position:** a range within that exact file. Different use positions
   can denote the same symbol; line/column is neither a unique name nor a durable ID.
3. **Snapshot identity:** a digest of the versioned canonical input inventory,
   exact source/manifest hashes, selected membership and lexical-to-physical path
   bindings, context, standard library, and analysis-affecting options/tool identity.
   Store-local revision counters and AST addresses are not cross-process identities.

The reference carries the defining file hash and context needed to reconstruct
the inventory; the response exposes that inventory for audit. Phase 0 must define
an operation-independent input scope so `find`, `inspect`, and `refs` can reuse
one reference. Hash that scope, not incidental lazy reads or their order. Expanding
the scope requires a fresh snapshot/reference. Define canonical serialization and
the analysis-contract/build fingerprint; package version alone cannot distinguish
development builds. Result limits and formatting do not change symbol identity.

On reuse, reconstruct the same scope, compare snapshot and declaration hashes,
then re-resolve and verify identity and span. Reject changes to any participating
source, manifest, membership, library, path binding, or analysis contract. Do not
fall back to a similar name, moved span, or a fresh search. References are local
to the checked source tree, not portable across arbitrary repository moves.
Require complete, verified snapshot identity before issuing a reusable reference;
partial observations may still carry locations but cannot silently become edit
authorization. References are validation data, not filesystem access permission.

Use the existing frozen-input mechanism throughout discovery and analysis, then
verify bytes, membership, and identities before reporting. Never mix revisions
or retry silently. The verification is a checked snapshot, not atomic isolation;
a later mutation must perform its own checks immediately before publication.

### Bounds and deterministic output

Retain the existing read deadline and hard input/JSON ceilings initially. Expose
effective limits even on failure. Apply collection limits during traversal or
bounded selection, not only after building unbounded vectors. Proposed defaults:

| Operation/data | Default | Hard maximum |
| --- | --- | --- |
| `find --limit` | 100 symbols | 10,000 |
| `refs`, `callers`, `callees --limit` | 1,000 entries per named relation collection | 10,000 per collection |
| `inspect` | 32 KiB aggregate source excerpts; 12 supporting types | 256 KiB; 64 types |
| Documentation/signature text; one-line previews | 4,096 scalars; 160 scalars | Existing limits |

Initially keep inspection budgets fixed and reported; configurable spellings are
an open UX decision. Count UTF-8 excerpt bytes and JSON escaping separately, and
clip excerpts only at valid character boundaries.
Explicitly report omitted sections, clipped text, and unknown totals. Never clip
an edit's replacement bytes and then publish it; oversized mutation plans fail
before publication. Do not truncate target identities into usable references.

Rank `find` by exact short or qualified match, short prefix, short substring,
qualified prefix, then qualified substring. Tie-break by canonical qualified
name, project identity, canonical file, byte offset, and kind; preserve original
spelling in output. Existing workspace-symbol ranking needs adaptation for exact
qualified matches and coverage. Bounded top-N selection must not depend on hash-map
iteration or file discovery order. If discovery itself is interrupted, label
ranking and coverage partial; a missing exact match cannot be ruled out.

Other locations sort by reported path and byte offset, with explicit relation
kind/target tie-breaks. Deduplicate by physical declaration/use span and semantic
context; a file interpreted differently in two projects is not silently merged.
Report memberships when equivalent interpretations are merged. Keep multiple
call sites between the same pair of routines. Diagnostics, reasons, inventories,
and omitted sections have stable ordering too. Identical inputs/options produce
identical semantic JSON; omit wall-clock timestamps and random IDs from it.

## Prioritized implementation phases

### Phase 0: common infrastructure and existing-query convergence

Define typed service request/results, schema-2 fixtures, selectors, snapshot
references, coverage categories, and bounded serialization. Migrate existing
read queries and rename's interface without weakening rename checks. Reuse
structured diagnostics and cancellation rather than adding parallel CLI logic.

Move rename preparation onto the shared frozen-input/context contract, including
manifests and membership; retain before/after overlay analysis. The hard read
worker deadline must not be copied blindly to an in-progress multi-file commit.
Keep publication owned by the surviving CLI coordinator: cancellation/deadlines
can abort preparation or pre-publication checks; once replacement begins, finish
publication or restoration and report its state. This bounded-analysis versus
uninterruptible-publication distinction must be explicit in help and `limits`.

**Acceptance:** existing navigation and rename safety fixtures pass under the new
schema; failures produce structured outcomes; no program executes or read query
writes source/build artifacts. Validate wrong context, invalid selectors,
source/manifest/membership changes, stale references, timeout, output failure,
permissions, staging failure, commit failure, and failed restoration. Publication
results must identify unchanged, replaced, restored, and unresolved paths.

### Phase 1: `find` and declaration selection

Expose symbol discovery from the language service with name, qualified name,
kind, signature where available, source location, short declaration documentation,
origin, owner/project, and analysis state. Reuse declaration spans and documentation
extraction; absent documentation is absent, not an invented summary. Syntax-only
candidates may be returned but must be labeled and cannot establish a unique
checked target. Extend existing queries with the selectors defined above.

Search declarations owned by selected projects, including their private/local
symbols as source navigation, and eligible dependency exports. Label visibility
and origin. Being discoverable never makes a private symbol callable by another
project. Dependency-private implementations remain outside consumer discovery;
select their owning project to inspect them. Do not index the entire installed
Std implementation by default; eligible imported Std APIs and source-less builtin
declarations are distinguishable, with null physical locations where appropriate.

**Acceptance:** exact matches precede partial matches; duplicate short names and
same qualified names in different projects remain ambiguous. Test case differences,
aliases, shadowed locals, generic source declarations, dependency exports, excluded
files, builtins, and repeated deterministic runs. A reduced limit reports omissions
without selecting the first candidate. Position and reference selection of the
same declaration agree; expression `type` behavior remains intact.

### Phase 2: bounded `inspect` context

Return sections in this order: selected declaration signature/documentation,
implementation if available, then directly relevant project types. Each section
has a reason for inclusion, owning symbol, exact source range/hash, origin,
analysis state, and `excerpt`/`truncated` flags. Keep declaration and implementation
locations distinct where necessary. A missing body is explicitly unavailable,
for example for a builtin or declaration-only dependency; never synthesize one.

Use the compiler-linked declaration/AST for implementation bounds. Supporting
types are direct nominal types in the signature, receiver, and explicit local
type annotations, in that priority and source order. Unwrap structural wrappers
such as arrays to identify their directly named type arguments, deduplicating
semantic identities. Include the named type or alias declaration once; do not
recursively expand its members' types, callees, imports, or method bodies.
Library-private details follow the selected context's visibility policy.

For large records/enums, return labeled source excerpts such as the header and
selected member declarations with their separate ranges; expose omitted members
and known counts. A set of excerpts is not a complete compilable declaration.
If a routine body exceeds the budget, return clearly labeled partial ranges and
the full body's location/size. Do not splice disjoint source into apparently
continuous code. Missing checked type links yield explicit unavailable context,
not a guessed type from a name or hover string.

**Acceptance:** generic signatures and imported aliases resolve to original
declarations; supporting types follow the stated one-hop rules. Recursive types,
large bodies, nested routines, and unavailable implementations stay bounded.
All excerpts reproduce exact snapshot bytes, including Unicode and line endings.
Absent docs, deliberate omissions, budget clipping, and failed analysis remain
distinguishable. Inspecting one function never recursively includes its callees.

### Phase 3: direct `callers` and `callees`

Build source relationships from checked AST and compiler resolution metadata in
the language service. Existing references alone cannot classify calls. If the
compiler does not retain a needed binding, add snapshot-scoped metadata at its
existing resolution point; do not infer a target from matching names, argument
counts, hover strings, or debugger bytecode function IDs.

Each relation records its kind, containing callable, target when proven, exact
call/use location, one-line source preview, and resolution state. A source generic
routine is one declaration; checked type arguments may annotate a use but do not
create invented source declarations for each compiled specialization. Resolve
aliases, shadowing, methods, and imported declarations through their original
semantic identities. Attribute nested closure bodies to that closure, with a
snapshot-bound span identity when anonymous, not to its enclosing routine. Use
an explicit owner kind for program entry or unit initialization.

| Relationship | Meaning and reporting |
| --- | --- |
| `direct_call` | A call expression/statement with a compiler-proven callable target; returned in `calls`. Static recursion is an ordinary direct edge. |
| `function_reference` | Taking or storing a callable value; returned in `related_uses`, never counted as a call. |
| `callback_argument` | Passing a callable to another call, with receiving callee/parameter when checked; the receiving call is separate. This does not prove the callback executes. |
| `dynamic_call` | Calling a callable parameter, variable, field, returned value, or other target not proven statically; returned in `unresolved_sites` with a reason and null exact target. |

Call syntax alone does not guarantee a user-routine edge: distinguish constructors
and intrinsic operations using compiler information. Report builtin calls with
their known identity and origin, without inventing an implementation. A task
spawn of a proven routine may be a direct edge tagged `invocation: spawn`;
ordinary calls use `invocation: ordinary`. Lowering details such as tail-call
opcodes do not replace source call sites. Recognizing additional callable-value
targets requires proven data-flow metadata, not heuristic propagation.

`callees` reports the selected body's direct calls, related callable uses, and
unresolved call sites. `callers` reverses proven direct edges and lists known
references/callback passages separately. It also reports unresolved sites in the
eligible analyzed context that could not be ruled out as potential uses; it must
not attach them as proven callers of the selected routine. Initially a conservative
scope-wide unresolved inventory is acceptable. Narrowing it requires compiler
proof, not name matching. Apply and report separate collection counts/limits.

Expose `direct_edges_complete` and `runtime_targets_complete` in relation coverage.
Relevant unknown targets make the overall result incomplete even when all proven
direct edges were enumerated. Zero direct callers is not proof of zero possible
runtime calls. Analysis failures, omitted bodies, and external consumers are
reported with their distinct scope limitations. Completeness is always relative
to the declared context; no query claims knowledge of outside consumers.

**Acceptance:** direct recursion, mutually recursive routines, aliases, shadowing,
generic calls, methods, imports, and multiple call sites resolve correctly.
Passing a routine as callback yields a related use, not a fabricated direct call
to it. Calling a stored routine value remains unresolved unless binding is proven.
Test nested/anonymous closures, callable fields, spawns, builtins, initializers,
broken sources, export boundaries, and empty direct-call lists with dynamic uses.
No unresolved or unsearched region may disappear into a complete empty answer.

### Phase 4: later checked parameter refactoring

This phase is outside the initial read-query implementation and needs its own API
decision. The proposed operation is: add a parameter to a selected routine and
adapt its known call sites. Do not publish a CLI spelling until its edit-policy
and unsupported cases are decided.

Require an explicit parameter name/type/mode and an explicit argument policy.
First support ordinary, fully resolved calls; unknown dynamic uses, callable-value
references, callbacks, unsupported method/constructor cases, and generic cases
without sufficient metadata block application with locations and reasons. Do not
silently edit direct calls while leaving incompatible function values behind.
Exported routines require an explicit statement that only selected consumers are
covered, never a claim of whole-ecosystem safety.

Generate a deterministic preview tied to the starting snapshot, including every
edit, old text, and affected source hash. Check identifier/parameter conflicts,
named/positional arguments, `var` storage requirements, types, and evaluation
order. A caller-supplied new argument can introduce effects: do not invent a
default expression or claim behavioral equivalence merely because it type-checks.
Analyze all known affected projects and consumers against in-memory overlays
before publishing. Application rechecks the exact preview/preconditions rather
than silently recomputing a different plan. Incomplete analysis blocks mutation.

Reuse the staged publisher and strengthen its input checks as established in
Phase 0. Keep per-file replacement and best-effort restoration guarantees explicit.
Report publication state even on failure; a broken output stream or killed process
requires inspecting/re-querying saved files before retrying. Do not promise atomic
multi-file edits or automatic safe retry.

**Acceptance:** supported calls and their declaration type-check after overlay
edits; preview is reproducible and application matches it. Conflicts, stale files,
dynamic uses, missing consumers inside the selected scope, analysis errors, and
unsupported constructs block writes. Failure-injection tests cover staging,
replacement, rollback, and reporting; behavioral regression tests cover evaluation
order and mutable argument storage for every supported transformation.

## Architecture and reuse

Follow the [Rust architecture](../rust-architecture.md):

- `fpas-parser` supplies recovered ASTs and exact source spans; `fpas-sema` owns
  bindings, checked types, generic substitutions, and any missing call metadata.
  Preserve metadata's association with the original snapshot AST allocation.
- `fpas-project` owns manifest discovery, dependency/export rules, `UnitGraph`,
  and frozen inputs. Extend its snapshot inventory where needed; do not reproduce
  project selection or path-containment rules in new CLI commands.
- `fpas-language-service` owns reusable query semantics, symbol selection,
  inspection/context rules, source relations, and edit planning. Reuse snapshots,
  cancellation, diagnostics, navigation, and analysis caching. Use typed results
  so CLI and later editor clients consume the same meaning.
- `fpas-cli` parses selectors, coordinates deadlines, renders bounded JSON, and
  publishes checked edits. It performs no independent name or call resolution.
  `fpas-lsp` retains protocol/position conversion and can later adapt shared APIs.
- `fpas-code` retains debugger bytecode effect analysis. These source queries
  introduce no dependency on VM execution, native host operations, or runtime
  values and do not require compiling the inspected program.

Start with derived, request-local indexes over existing analyses. Reuse the
running service's cache and cancellation checkpoints. Separate CLI invocations
still reload and analyze; a snapshot digest is not itself a persisted analysis.
Consider a persistent index or background service only after measuring cold and
warm query costs and identifying a material bottleneck. Any later cache must
validate manifests, library selection, membership, source bytes, and compiler
contract, with the same correctness as an uncached query.

## Cross-phase verification and benefit measurement

Extend service tests for resolution and coverage, CLI tests for schemas and exit
codes, and filesystem tests for snapshot/publication behavior. Reuse the existing
[navigation](../../crates/fpas-language-service/tests/navigation_resolution.rs),
[cache](../../crates/fpas-language-service/tests/query_cache.rs),
[rename safety](../../crates/fpas-language-service/tests/rename_safety.rs), and
[CLI query](../../crates/fpas-cli/src/main_tests/query.rs) fixtures.
Each phase must additionally pass this shared matrix:

| Case | Required observation |
| --- | --- |
| Unique/ambiguous names; aliases and lexical shadowing | Correct identity or explicit ambiguity; no first-hit selection. |
| Locals, imports, generic uses, several projects and library exports | Original declaration and consumer-specific visibility; external scope clearly excluded. |
| Parse/type failures, missing metadata, cancellation and limits | Proven partial facts only, incomplete state and explicit omissions. |
| Unicode, mixed line endings, BOM and non-boundary offsets | Exact byte/scalar correspondence; invalid coordinates rejected, no LSP UTF-16 leakage. |
| Stale references; edits during analysis; manifest/glob/symlink changes | Explicit rejection/incompleteness, never mixed snapshots or automatic retargeting. |
| Bounded search, context, calls and JSON | Deterministic retained results with per-collection counts and omission reasons. |
| Repeated runs and reordered discovery | Same semantic output and deduplication for identical snapshots/options. |
| Dynamic callables and callbacks | References distinct from calls; uncertain targets remain visible. |

After correctness, evaluate realistic FPAS tasks: locate an unfamiliar API,
explain a routine using its types, enumerate known callers across projects, and
make a validated change. Compare existing tools (text search plus positional
queries/editor navigation) with the extensions on identical repository snapshots,
task descriptions, selected contexts, and validation criteria. For automated
agents hold model, instructions, budgets, and available validation tools constant;
repeat tasks and alternate order to limit learning/cache effects. Include failures
and incomplete answers, not only successful examples.

Measure correctly completed tasks, missed or misattributed uses, correction
rounds, tool calls, transferred context bytes and tokens, and total elapsed time
including tool execution. Record tool latency separately, cold versus warm service
state, output limits, environment, and token-counting method. Validate final edits
with the same compiler/tests and independently reviewed expected usages. Store
experimental reports under `.temp-data`, not as claims of established productivity.
No productivity improvement or reliable test-selection benefit has been measured
for this proposal.

## Open decisions and exclusions

Before Phase 0, finalize canonical reference serialization and tool fingerprint,
schema fixtures, publication-state fields, and cancellation behavior at the write
boundary. The required rejection and recovery guarantees above are not optional.
Before Phase 2, decide whether to expose inspection-budget flags. Before Phase 3,
audit available call metadata and enumerate the exact supported source forms;
keep unsupported forms explicit. Phase 4 needs its argument-policy and review/apply
interface decisions. These decisions do not require a language syntax change.

Excluded from the initial scope: a primary graph store or graph database, a new
source representation, persistent node IDs across arbitrary edits, program
execution for discovery, automatic recursive context expansion, whole-program
points-to analysis, and cross-repository consumer discovery. Bounded transitive
calls can be considered after direct queries; a static graph alone cannot provide
complete change-impact analysis or reliable automatic test selection. A daemon,
persistent index, and broader refactorings require separate evidence and design.

The next implementation step is Phase 0's contract and fixture work, followed by
Phase 1. Keep this plan current as decisions are made; move implemented contracts
to the handbook only when code and tests provide them.
