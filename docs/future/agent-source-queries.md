# Source queries for coding agents

Status: delivered. [AP56](improve-syntax/ap56-source-queries/README.md) implements all five
phases on branch `ap56`: foundation, find, inspect, direct call queries and
checked refactoring. Current user contracts are in the handbook. Agents decide
technical and CLI contracts autonomously; only FPAS syntax changes require user
approval. This plan requires no FPAS syntax or execution change.
The [adapter contract](improve-syntax/ap56-source-queries/adapter-contract.md)
requires the shared analysis core and both CLI/LSP connections in each delivery.

## Problem and goal

Agents can already ask semantic questions at a saved source position. Finding
that position, collecting a useful implementation context, and separating calls
from other references still require several operations and manual interpretation.
The goal is a bounded, reproducible analysis core whose typed facts and checked
edits are used by both CLI and LSP, including unsaved editor snapshots.

FPAS source remains the authoritative program representation. Symbols, types,
relations, and edit plans are derived from exact source snapshots and compiler
analysis. They do not constitute a new source format or a primary graph store.

Priority is: name discovery and selection, focused inspection, then direct call
relationships. Adding a parameter and updating known calls comes after the
read-only tools. Existing rename provides useful infrastructure, not proof that
arbitrary refactorings are safe.

## Planning baseline

The implemented user contracts are documented in [source queries](../pascal/tools/source-queries.md),
[editor integration](../pascal/tools/editor-integration.md), and
[diagnostics](../pascal/tools/diagnostics.md). The following baseline at `a5f65fcdb`
motivated the additions delivered by AP56.

| Area | Present behavior and source evidence |
| --- | --- |
| CLI selection | [`cli_input/query.rs`](../../crates/fpas-cli/src/cli_input/query.rs) accepts `refs`, `definition`, `type`, and `rename`. All require one `file.fpas:line:column`; there is no name selector, `find`, `inspect`, `callers`, or `callees`. |
| Read-only queries | [`request.rs`](../../crates/fpas-cli/src/cli_query/request.rs) supervises a worker and handles parser errors as JSON. [`worker.rs`](../../crates/fpas-cli/src/cli_query/worker.rs) calls [`LanguageService::source_query`](../../crates/fpas-language-service/src/navigation/source_query.rs). Read queries neither compile bytecode nor execute FPAS. |
| Read contract | [`report.rs`](../../crates/fpas-cli/src/cli_query/report.rs) builds schema-1 JSON with `serde_json::Value`, including diagnostics and coverage. A typed schema-2 wire model is new work. `request::selected` explicitly bypasses rename, whose dispatcher and error path remain separate. |
| Context and snapshots | [`SourceInputs`](../../crates/fpas-project/src/source_inputs.rs) freezes bytes and verifies lexical/physical aliases and glob membership. Its guard is thread-local and exports files/reasons, not a reusable owned inventory of every scope input. The worker hashes files read during that operation; this is not an operation-independent snapshot or target reference. |
| Name search | [`workspace_symbols.rs`](../../crates/fpas-language-service/src/navigation/workspace_symbols.rs) ranks case-insensitive matches, collects all matching declarations, then truncates at 100 without coverage. [`analysis/service.rs`](../../crates/fpas-language-service/src/analysis/service.rs) builds that index from parser symbols and includes all editor API sources. It is not a checked, visibility-filtered, bounded `find` implementation. |
| Navigation metadata | [`DocumentSymbol`](../../crates/fpas-language-service/src/symbols/document.rs) has identity, spans, visibility and callable detail. [`DocumentAnalysis`](../../crates/fpas-language-service/src/analysis/document.rs) retains checked metadata on the original AST; [`AnalysisMetadata`](../../crates/fpas-sema/src/check/context.rs) already has declaration types and binding references. Reuse these instead of parsing hover text. |
| Intrinsic declarations | [`StandardLibraryContext`](../../crates/fpas-language-service/src/workspace/standard_library.rs) identifies generated `lib/api/Std` sources. Their FPAS bodies are declaration stubs, not native implementations. Existing API lookup shares documentation extraction but excludes routine locals and does not provide source-query completeness. |
| Analysis reuse | [`DocumentSnapshot`](../../crates/fpas-language-service/src/document/snapshot.rs) owns immutable text, parsed AST, line index, and store-local revisions. [`AnalysisCache`](../../crates/fpas-language-service/src/analysis/cache.rs) keys completed analyses by participating revisions and project context; request forks share it within a running service. Separate CLI worker processes do not share that cache. |
| Rename | The former `cli_query/rename.rs` pinned known source snapshots, checked analysis before and after in-memory edits, checked conflicts through the service, and supported `--dry-run`. Its `publication.rs` staged replacements and verified source bytes before publishing. |
| Other refactorings | [`refactoring/mod.rs`](../../crates/fpas-language-service/src/refactoring/mod.rs) offers checked extract-variable and organize-imports editor actions. They have no CLI plan/apply interface. Reuse their evaluation-order and initializer-effect restrictions. |
| Call metadata | [`AnalysisMetadata`](../../crates/fpas-sema/src/check/context.rs) already retains binding references, named-argument ordering, method/fluent/intrinsic targets and constructor/conversion classifications. It does not expose a complete source-call inventory or parameter-change engine. |
| Unit dependencies | [`UnitGraph`](../../crates/fpas-project/src/unit_graph/model.rs) retains parsed units, origins, direct `uses`, and source identities. Import reachability is not a function-call graph. |
| Debugger effects | [`fpas-code/src/debug/effects.rs`](../../crates/fpas-code/src/debug/effects.rs) classifies verified bytecode, follows direct call/tail-call edges to transitive effects, and marks dynamic calls and callback intrinsics conservatively. A general source-call query still requires source-level relationships. |
| Tool identity | [`compiler_identity.rs`](../../crates/fpas-build/build/compiler_identity.rs) hashes compiler/artifact inputs, excluding the CLI and language-service crates. It cannot alone identify this query implementation. No query-specific build fingerprint or canonical target-reference serialization exists. |

The units-basic example resolves `App.Math.Clamp` through schema 1; `query find`
is an invalid request. AP56 retains the semantic results while adding checked
selection and adapter mappings. Work files own paths, migration and acceptance;
the phases below are not a second completion index.

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
fpas refactor rename src/client.fpas:12:5 --name Submit --context dungeon.fpasworkspace --out rename.plan.json
fpas refactor apply --plan rename.plan.json
```

`find` takes search text. Declaration operations accept exactly one of a position,
`--symbol <name>`, or `--target-ref <json-file>`. Reject mixed selectors, missing
values, repeated options, and unsupported operation options through the common
error envelope. Rename also requires `--name`. Range/file refactorings use the
selectors specified in AP56.6. Keep positional
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

Use a common `schema_version: 2` envelope for queries and refactoring commands.
Keep one UTF-8 JSON object plus newline on stdout for success and failure;
reserve stderr for inability to deliver the final report. Help remains help.
The exact operation payloads must have schema fixtures before implementation.
Preparation distinguishes a checked no-change result from an unsupported action
or incomplete analysis; absence of an editor action is not evidence of no change.

| Field | Proposed meaning |
| --- | --- |
| `schema_version`, `operation`, `outcome`, `exit_code` | Version, recognized operation or null, result class, matching process exit. |
| `request` | Tagged selector or search text, operation arguments, and effective options; unavailable parsed fields are null. |
| `context` | Kind, manifest, sorted projects, standard-library selection, path base, and searched scope. |
| `snapshot` | Content-derived request identity, analysis-contract identity, and hashed input inventory; null if unavailable. |
| `target`, `candidates` | Selected declaration and reusable reference, or bounded ambiguity candidates. |
| `result` | Tagged operation payload: symbols, locations, type, inspection sections, relations, or refactoring plan/publication information. |
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

Limit reference-file reads/decoding to the shared 8 MiB public ceiling. Resolve
recorded identity paths from their original context/path base, independent of
the new invocation's working directory; display paths remain presentation only.

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
Require a complete, verified input scope and checked declaration before issuing
a reusable reference. AP56.1 distinguishes that scope verification from a clipped
result collection; clipping cannot establish short-name uniqueness. Partial
analysis may carry locations but no reusable references. References are validation
data, not filesystem access permission; a mutation repeats complete validation.
AP56 uses references only for source-backed checked declarations.
Source-less builtins keep their known identity with a null reference and explicit
reason; generated API declarations with real spans may still have references.

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
| `inspect` | 32 KiB aggregate source excerpts; 12 supporting types | Same fixed budgets in AP56 |
| Documentation/signature text; one-line previews | 4,096 scalars; 160 scalars | Existing limits |

AP56 uses fixed, reported inspection budgets without new budget flags.
Count UTF-8 excerpt bytes and JSON escaping separately, and
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

[AP56.1](improve-syntax/ap56-source-queries/01-contract.md) specifies schema,
snapshot/reference and publication fixtures for the recorded technical choices.
[AP56.2](improve-syntax/ap56-source-queries/02-foundation.md) owns delivery and
migration of existing queries. Preserve semantic results, edit checks and
attributed diagnostics. Worker preparation is bounded; the surviving coordinator
finishes publication or ordinary recovery once replacement begins. Oversized
edit bytes and identity fields are rejected before publication, never clipped.

### Phase 1: `find` and declaration selection

[AP56.3](improve-syntax/ap56-source-queries/03-find.md) owns checked discovery,
bounded ranking and position/name/reference selection. Require nonempty find
text. Respect owned/private versus dependency/export scope, label origins and
incomplete analysis, and preserve declaration identity across selectors. Existing
editor workspace search is a reuse source, not proof of complete discovery.
Its current unbounded collect-then-truncate path cannot satisfy this contract.
Preserve its empty-pattern, open-document and unimported API discovery through
the adapter contract's explicit `editor_catalog` policy. CLI and rich extension
find default to `checked_context`; catalogue-only results grant no portable
reference, uniqueness or edit eligibility.

### Phase 2: bounded `inspect` context

[AP56.4](improve-syntax/ap56-source-queries/04-inspect.md) owns ordered declaration,
implementation and direct-type context, exact excerpt provenance, fixed budgets
and omission reporting. Retain contract clauses and original checked AST identity.
Generated editor API bodies are declaration stubs, never native implementation.
Source-backed Std routines remain inspectable under their visibility policy.
Supporting types are one-hop checked identities; no recursive callee/import
expansion or semantic inference from display strings is permitted.

### Phase 3: direct `callers` and `callees`

[AP56.5](improve-syntax/ap56-source-queries/05-call-relations.md) owns delivery,
the source-form audit, affected paths and regression coverage.

Build source relationships from snapshot-owned checked AST/retained expression
allocations and compiler resolution metadata in
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
Record-field defaults have a declaration/default owner: they execute in their
declaration environment when construction needs the default. Traverse checked
`record_defaults` and `static_bound_declarations` allocations with their retained
metadata and deduplicate original/retained source sites. Do not multiply a default
call by its construction sites or misattribute it to unit initialization.

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
The latter describes possible targets of inventoried source sites, not all runtime
effects. Report compiler-generated invocation exclusions, such as resource cleanup;
refactoring must still account for compiler-bound uses of the changed declaration.
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
record defaults and retained static-bound expressions,
broken sources, export boundaries, and empty direct-call lists with dynamic uses.
No unresolved or unsearched region may disappear into a complete empty answer.

### Phase 4: checked refactoring and parameter insertion

[AP56.6](improve-syntax/ap56-source-queries/06-refactor.md) owns the shared
`fpas refactor` preview/plan/apply interface, migrating rename and exposing the
existing extract-variable and organize-imports transformations.
[AP56.7](improve-syntax/ap56-source-queries/07-add-parameter.md) appends an explicit
parameter and adapts proven supported calls. These are agent-owned CLI and
implementation decisions; emitting existing FPAS syntax needs no grammar approval.
Proposed commands are documented in those work files and stay out of installed
help until implemented. Remove `query rename` when its canonical replacement ships.

Require an explicit parameter name/type/mode and one caller-supplied argument
expression, checked independently in every caller's scope. Append only; parameter
removal/reordering and per-call expression policies are outside this operation.
Use AP56.7's fixed matrix: non-generic named routines and ordinary direct calls,
including checked non-generic nested routines and record defaults. Generic bodies,
methods/fluent calls, spawns, callable-value/callback uses and unresolved relevant
calls block application with locations and reasons. Parse complete type/argument
fragments, reject trailing tokens and validate the type in the declaration's scope.
Compiler-bound implicit uses also block unsupported transformations. Do not
silently edit direct calls while leaving incompatible function values behind.
Exported routines require an explicit statement that only selected consumers are
covered, never a claim of whole-ecosystem safety.

Generate AP56.6's deterministic plan tied to the starting snapshot, including the
recipe, complete before/after inventories, every edit, old text and source hash.
Only explicit `apply --plan` writes source. Check identifier/parameter conflicts,
named/positional arguments, `var` storage requirements, types, and evaluation
order and unchanged binding identities. A new argument can introduce effects;
do not invent a default or claim equivalence merely because it type-checks.
Analyze all known affected projects and consumers against in-memory overlays
before publishing. Application rechecks the exact preview/preconditions rather
than silently recomputing a different plan. Incomplete analysis blocks mutation.
Validate plan structure/context first, then compare full inventories before
resolving the pre-edit target. Matching expected-after state reports `ok` with
`result.status: already_applied` and no writes; an eligible empty plan reports
`no_change`. A partial state is `stale_plan`. Bound artifact decoding and keep
identity paths independent of the applying process's current directory.

Standard editor extract/import actions carry a snapshot-bound recipe command,
not a cached edit. Invocation revalidates all relevant inputs before requesting
editor application. Standard rename checks in its final request. Client rejection
is reported; the adapter contract documents the remaining delivery/application
race for unedited dependencies and closed files.

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
  Preserve metadata's association with its snapshot-owned original or retained allocation.
- `fpas-project` owns manifest discovery, dependency/export rules, `UnitGraph`,
  and frozen inputs. Extend its snapshot inventory where needed; do not reproduce
  project selection or path-containment rules in new CLI commands.
- `fpas-language-service` owns reusable query semantics, symbol selection,
  inspection/context rules, relations, edit planning and semantic revalidation.
  Reuse snapshots, cancellation, diagnostics and analysis caching. Both adapters
  must call these operations; reusable checks move out of CLI-only rename logic.
- `fpas-cli` parses selectors, coordinates deadlines, renders bounded JSON, and
  publishes checked edits. It performs no independent name or call resolution.
- `fpas-lsp` maps standard requests and the required query/refactoring extensions
  to that same core, with synchronized overlays and versioned editor edits.
  The adapter contract fixes required editor commands and parity/overlay tests.
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

## Cross-phase verification

Extend service tests for resolution and coverage, CLI tests for schemas and exit
codes, and filesystem tests for snapshot/publication behavior. AP56's work files
name the primary owner for each regression. Reuse the existing
[navigation](../../crates/fpas-language-service/tests/navigation_resolution.rs),
[cache](../../crates/fpas-language-service/tests/query_cache.rs),
[rename safety](../../crates/fpas-language-service/tests/rename_safety.rs), and
[CLI refactoring](../../crates/fpas-cli/tests/refactor.rs) and
[CLI source-query](../../crates/fpas-cli/tests/source_queries.rs) fixtures.
The CLI integration cases run under target `tools`; semantic cases run under
language-service `navigation_edits`, with cache checks under `analysis_state`.
Register new case modules explicitly; both crates disable automatic integration
test discovery. Compile tests separately from execution as the build guide requires.
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

Use fixed source fixtures for unfamiliar API lookup, declaration/type context,
known callers and checked edits. Assert exact expected identities, omissions,
blockers and post-edit compiler behavior through the owning tests above. Include
failure and incomplete cases. No LLM task benchmarks, repair-count comparisons or
model-based acceptance gates belong to AP56. Tool latency can be measured
separately if an observed performance problem warrants it. No productivity or
automatic test-selection benefit is claimed.

## Open decisions and exclusions

AP56.1 specifies canonical encoding, CLI/LSP contracts and core build inputs.
AP56.2 generates the shared analysis-core fingerprint, independent of adapter
executables; source/library hashes and editor version guards remain distinct.
AP56.5 audits call forms; AP56.7 maps its fixed supported/blocked forms to metadata
and regression owners.
Agents resolve these engineering details autonomously. Rejection and recovery
guarantees remain required; no FPAS syntax change is needed.

Excluded from the initial scope: a primary graph store or graph database, a new
source representation, persistent node IDs across arbitrary edits, program
execution for discovery, automatic recursive context expansion, whole-program
points-to analysis, and cross-repository consumer discovery. Bounded transitive
calls can be considered after direct queries; a static graph alone cannot provide
complete change-impact analysis or reliable automatic test selection. A daemon,
persistent index, and broader refactorings require separate evidence and design.

All five phases are delivered. The handbook owns the current
[source-query](../pascal/tools/source-queries.md) and
[refactoring](../pascal/tools/refactoring.md) contracts.
