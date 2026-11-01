# AP56: Shared analysis core, source queries and refactoring

Status: Complete. Priority: P1. Effort: XL.
Branch: `ap56`, based on `main` at `a5f65fcdb`.
Completion is tracked in the [central plan](../README.md#package-status).

## Goal

Extend one analysis core used by both LSP and CLI. Find declarations, inspect
implementation/type context, enumerate direct calls and prepare checked source
changes through shared typed operations. Expose them as CLI queries/refactoring
plans and actual LSP/editor features, with common semantic identity and validation.

The [source-query detail plan](../../agent-source-queries.md) owns the proposed
cross-operation contract. AP56 owns delivery of phases 0 through 4. Read that
contract before implementing a work package here.
The [adapter contract](adapter-contract.md) defines the mandatory mapping,
source modes, shared build identity and completion checks for both consumers.

## Delivery state and target contract

The shared core now owns frozen inventories, build/snapshot identity, checked
find and position/name/reference selection, bounded inspection, and common
before/after edit validation. Read queries emit schema 2 through an isolated CLI
worker. Native `fpas/sourceQuery` routes use editor overlays, while standard
workspace symbols use the shared bounded collector with catalogue policy.
The registered VS Code inspection view passes the complete Extension Host suite,
including dirty dependencies and explicit stale-view rejection.

Direct call relationships now share checked original/retained AST traversal,
bounded CLI/rich-LSP projections and standard Call Hierarchy. Focused semantic,
CLI and hierarchy tests pass, including dirty and stale editor inputs. The
hierarchy tests also fixed inferred-context snapshots differing from their
recorded manifest reconstruction.

The core and CLI now prepare deterministic rename/extract/import plans, distinguish
no-change from unsupported/incomplete, and revalidate complete before/after states.
The canonical `refactor` family replaces query-rename and its separate publisher.
CLI lifecycle/safety tests cover immutable artifact output, stale
membership, Windows replacement failure/recovery and final stdout failure. LSP
recipe handles, command-backed extract/import actions and standard rename now
share those core checks. Parameter insertion now uses the full call inventory,
fresh-name argument validation and mapped before/after binding comparisons. Its
core matrix and CLI/LSP process checks pass. The complete VS Code Extension Host
suite also passes, including parameter preview, dirty multi-file application and
undo. The full Rust workspace, FPAS suite, documentation examples and editor suite
pass. Repository discovery excludes scratch trees while preserving explicitly
selected projects inside them.

The delivered result is one schema-2 envelope and `--context` contract, checked
snapshot-bound targets, deterministic `find`, bounded `inspect`, direct
`callers`/`callees` and `fpas refactor` plan/apply commands. Keep source
positions, semantic resolution, diagnostics, import/export visibility and rename
protection. Read queries produce no FPAS execution or source/build artifacts.

The refactoring scope is rename, existing extract-variable/organize-imports
operations, and appending an explicit parameter while adapting proven calls.
Excluded: arbitrary signature/body rewrites, persistent indexes or daemons,
cross-repository consumers, unsaved-editor CLI inputs, new FPAS syntax, runtime
changes and new public standard-library units.

## Authority

Agents decide architecture, internal APIs, data structures, CLI/JSON contracts,
budgets and implementation autonomously. Replace obsolete designs without a
compatibility layer. Only changes to FPAS syntax need the user's explicit
approval. Refactoring valid source using existing syntax does not change the
language grammar. This package requires no syntax change.
The plan/apply protocol is a tool workflow, not a human confirmation gate;
an agent may perform both steps within an assigned source-editing task.

## Decisions

| ID | Question | Decision, owner and rationale | Affected work packages |
| --- | --- | --- | --- |
| Q84 | Package scope and integration branch? | User selected `ap56` and extended planning to all five detail-plan phases, including refactoring. | AP56.1 through AP56.7 |
| Q85 | Schema, selectors and reference identity? | Agent choice: schema 2 for CLI, typed shared semantic contracts, manifest contexts, SHA-256 canonical inventories and a shared analysis-core build fingerprint; invalidate stale references instead of retargeting. | AP56.1 through AP56.7 |
| Q86 | Preparation, publication and limits? | Agent choice: bounded isolated preparation, 128 MiB internal original/replacement budget, 8 MiB public JSON, coordinator-owned publication/recovery and exact edit bytes. | AP56.2, AP56.6, AP56.7 |
| Q87 | Inspection scope and budgets? | Agent choice: fixed 32 KiB excerpts and 12 direct supporting types; generated API stubs are declarations, not implementations. | AP56.4 |
| Q88 | Call relationships and completeness? | Agent choice: compiler-proven direct edges, separate value/callback uses and unresolved sites, with explicit direct-edge/runtime-target coverage. | AP56.5, AP56.7 |
| Q89 | Canonical refactoring interface? | Agent choice: `fpas refactor` prepares immutable plans by default; explicit `apply --plan` revalidates exact edits. Check the complete expected-after state before resolving an old target on retry. Distinguish no-change, unsupported and incomplete. Migrate rename into that family and remove `query rename`. | AP56.6, AP56.7 |
| Q90 | First parameter-change operation? | Agent choice: append one explicit value or var parameter to non-generic named routines, including non-generic nested routines, and adapt ordinary direct calls. Parse complete type/argument fragments; use AP56.7's fixed supported/blocked matrix, including record defaults. | AP56.7 |
| Q91 | Shared core and delivery of both adapters? | User selected one common analysis core with mandatory LSP and CLI integration. Agent choice: extend `fpas-language-service`; preserve editor catalogue discovery as an explicit policy; revalidate command-backed extract/import actions at invocation; keep transport/publication in adapters under `adapter-contract.md`. | AP56.1 through AP56.7 |

Only this table records decision outcomes. Work files refer to these IDs.

## Contract details

The [wire contract](wire-contract.md) records schema/plan shapes, canonical-encoding
vectors and exact diagnostic
and publication fields, including LSP extension contracts and core build inputs.
AP56.5 records the source-form coverage audit; AP56.7 maps metadata and regression
owners to its fixed edit-site matrix. These details implement the recorded choices.
There is no pending FPAS syntax decision.

## Dependencies and work packages

No other unfinished AP is a prerequisite. Use the existing parser, semantic
analysis, project loader, frozen-input guard, language service, worker supervisor
and staged publisher.
The user selected `ap56` as this package's integration branch; completed
prerequisites on that branch may be used by the next work package.

- [x] [AP56.1: Specify the common contract and fixtures](01-contract.md)
- [x] [AP56.2: Build the shared core and migrate both adapters](02-foundation.md)
- [x] [AP56.3: Add checked find and declaration selection](03-find.md)
- [x] [AP56.4: Add bounded inspect](04-inspect.md)
- [x] [AP56.5: Add direct call relationships](05-call-relations.md)
- [x] [AP56.6: Expose checked refactoring plans and apply](06-refactor.md)
- [x] [AP56.7: Add checked parameter insertion](07-add-parameter.md)

This is the only work-package checklist for AP56. Follow
[status tracking](../development-process.md#status-tracking). Each implementation
step includes its consumers, documentation and verification before completion.
The CLI, LSP and required editor connections in the adapter contract are part of
each feature's delivery; they are not deferred to a later integration package.
AP56.3 and AP56.5 depend on AP56.2; AP56.4 depends on AP56.3; AP56.6 depends on
AP56.2/AP56.3; AP56.7 joins AP56.5/AP56.6. The list is the recommended delivery
order, not an extra dependency between otherwise independent work packages.

## Acceptance

1. Existing positional queries and rename use the selected schema/context contract
   with preserved semantic results, visibility and edit protections.
2. `find` discovers eligible declarations deterministically, reports ambiguity
   and omissions, and never turns incomplete discovery into a unique target.
3. A checked source-backed target can be reused by `inspect` and existing queries
   only while its complete scope, sources, library and analysis identity match.
   Source-less builtins have explicit unavailable-reference/body reasons.
4. `inspect` returns exact, bounded declaration/implementation/type context.
   Generated intrinsic stubs are never presented as native implementation code.
5. Direct call queries distinguish proven edges, value/callback uses and unresolved
   targets. Incomplete analysis never implies zero possible callers.
6. Rename, extract-variable and organize-imports use shared plan/apply validation.
   Add-parameter adapts every proven supported call or publishes no source change.
   Reuse preserves existing transformations' effects, ordering and visibility.
7. Rejected inputs, source changes, incomplete analysis, deadlines and publication
   failures produce structured outcomes. Oversized edits fail before
   publication; ordinary recovery identifies any unresolved paths.
8. Current editor navigation keeps its behavior. No program executes to discover,
   plan or apply source edits. No new runtime or compiler-artifact format is introduced.
9. Every delivered operation is used through both actual adapter routes listed
   in the adapter contract. Equal snapshots/context/policy yield equal semantic
   facts; LSP also handles unsaved multi-file overlays and versioned edits.
   Core-only or CLI-only completion does not close a work package.

## Implementation and regression owners

| Owner | Responsibility |
| --- | --- |
| `fpas-project` | Source/manifest discovery, immutable inventory, aliases, membership and verification |
| `fpas-language-service` | Shared snapshots, build identity, selection, scope, coverage, find, inspect, calls and refactoring preparation/validation |
| `fpas-cli` | Arguments, typed wire rendering, process deadline, preparation transport and source publication |
| `fpas-lsp` | Direct core calls, standard requests/extensions, capabilities, document ordering and versioned editor edit projection |
| VS Code client | Inspect view and add-parameter command through LSP; existing navigation/call hierarchy/code-action UI |
| `fpas-sema` / `fpas-parser` | Existing snapshot-bound metadata and AST spans; extend only for a demonstrated missing binding |

Extend the existing CLI `tools` target's `source_queries` cases, CLI binary
rename/publication tests, and language-service `navigation_edits` and
`analysis_state` targets. Register new case modules in those existing targets;
automatic integration-test discovery is disabled. The work files name exact paths.
Rust test ownership is defined by the
[CLI](../../../../crates/fpas-cli/tests/README.md) and
[language-service](../../../../crates/fpas-language-service/tests/README.md) test guides,
with the [LSP target map](../../../../crates/fpas-lsp/tests/README.md) and existing
VS Code integration suite. Core tests prove semantics; adapter tests prove wiring,
conversion and lifecycle. The adapter contract names required parity/overlay checks.

## Documentation and verification

Update the implemented [source-query handbook](../../../pascal/tools/source-queries.md),
[CLI reference](../../../pascal/program-structure/cli.md), focused command help and
[agent tools reference](../../../pascal/agents/tools.md) with the delivering work
package. AP56.6 adds the refactoring handbook and migrates rename documentation.
Update editor integration, Rust architecture and extension documentation with the
delivering feature. LSP capabilities and editor commands must be usable and tested.
Keep unavailable commands in these future plans until then.

For Rust delivery, follow the [build guide](../../../bench/build-times.md#rust-test-workflow):
build programs with `dev-fast`/Thin-LTO and default Cargo parallelism, compile
tests separately, then execute the matching targets. Use the resource guard when
memory headroom is uncertain. Package completion includes the full Rust workspace
checks from the [development process](../development-process.md#required-checks).
Focused commands and the distinct regression risks belong to each work file.

Use fixed source fixtures to verify query facts, bounded context, exact edits
and explicit blockers. LLM task benchmarks, repair-count comparisons and
model-dependent acceptance gates are outside this package. Tool latency may be
measured separately if a concrete performance issue arises; no productivity
improvement is claimed. Apply the [shared constraints](../shared-constraints.md).
