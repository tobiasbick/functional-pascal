# Shared analysis core and mandatory adapters

This reference defines Q91's architecture and adapter acceptance for
[AP56](README.md). Both adapters implement this contract.
Each delivering work package includes its CLI and LSP connections; neither
adapter may be postponed to an unspecified later package.

## One semantic implementation

Extend `fpas-language-service` as the common analysis core. It already owns
snapshots, analysis caching, navigation and conservative refactorings, with no
LSP protocol dependency. A new core crate or a second resolver is unnecessary.
Keep parser, semantic rules and project discovery at their existing owners.

The core accepts explicit context, immutable source snapshots, search/analysis
policy, result budgets and cancellation. It returns typed facts, diagnostics,
coverage, target identity and checked edit plans. Name resolution, visibility,
call classification, supporting-type selection, conflict checks, whole-context
overlay validation and unchanged-binding checks belong here. Move reusable
before/after validation out of CLI rename into this shared implementation.

Adapters translate requests/results and own transport-specific effects. The CLI
loads saved sources, coordinates its worker/deadline, renders schema-2 JSON and
publishes files. LSP synchronizes editor documents, converts URI/UTF-16 positions,
maps standard requests/extensions and produces versioned `WorkspaceEdit` values.
The core neither writes sources nor depends on argv, CLI JSON envelopes, LSP
types, a running server or VM execution. LSP calls it directly, without spawning
the CLI. Core fixes and semantic extensions must reach both adapters.

## Source modes, identity and limits

CLI uses frozen saved input; LSP overlays all participating open FPAS documents
on project inputs. Saved manifests and configured workspace/library selection
retain their existing discovery rules. Overlay bytes must participate in
resolution, dependency analysis, coverage and edit validation, including changes
in another open file. Never silently analyze the disk copy of a dirty buffer.

Use a versioned build fingerprint generated for the shared core, covering its
analysis-affecting source/dependency inputs, manifests/lockfile, target/features
and contract version. The same core build has the same identity in both tools.
Do not hash the CLI or LSP executable into semantic identity. Keep library/source
hashes separate and preserve compiler-artifact identity unchanged. AP56.1 defines
canonical encoding and the exact tracked build inputs; AP56.2 implements them.

Equal source bytes, context, semantic policy and core build produce equal
semantic identities regardless of adapter. Editor revisions remain additional
local invalidation guards, not cross-process identities. Reusable references
require the complete verified inventory. Local editor handles may instead be
bound to the current session/snapshot and must be distinguished from portable
references. A reference to unsaved bytes fails as stale in a disk-only CLI until
that exact inventory is saved; no implicit save or fallback is allowed.

Normal editor requests may use narrower scopes and smaller budgets than strict
CLI discovery. These policies are explicit core inputs. Do not require a full
workspace inventory/hash for every hover. Equality checks compare equal policies;
intentional scope/budget differences remain documented. Changing a result limit
does not retarget a declaration. Cancellation and traversal bounds are shared;
the CLI's process deadline and JSON ceiling are adapter policies.

## Required operation mapping

| Work | CLI | Mandatory LSP/editor connection |
| --- | --- | --- |
| AP56.2 | Existing positional definition, refs, type and rename validation | Existing definition, references, type/hover and rename routes use the same checked selection/validation primitives, preserving editor revisions and caches. |
| AP56.3 | `find` and position/name/reference selection | `workspace/symbol` uses the shared bounded candidate collector; `fpas/sourceQuery` exposes rich find/selection results with coverage and ambiguity. |
| AP56.4 | `inspect` | `fpas/sourceQuery` operation `inspect`, plus a VS Code inspect command/view consuming those sections. Existing hover/type adapters reuse shared declaration/type facts without loading a whole inspection on every hover. |
| AP56.5 | `callers` and `callees` | Standard `textDocument/prepareCallHierarchy` and `callHierarchy/incomingCalls`/`outgoingCalls` use the same relation inventory; `fpas/sourceQuery` preserves unresolved/related-use collections and completeness. |
| AP56.6 | `refactor rename`, extract-variable, organize-imports and apply | Standard rename/extract/import code actions use shared preparation/validation. `fpas/refactor` exposes typed prepare/revalidate requests and returns editor edit projections. |
| AP56.7 | `refactor add-parameter` | A VS Code add-parameter command supplies explicit recipe fields to `fpas/refactor`; LSP prepares/revalidates through the same operation and returns versioned edits. |

`fpas/sourceQuery` and `fpas/refactor` are versioned JSON-RPC extensions, registered
and advertised by the LSP server with schemas fixed in AP56.1. They carry typed
selectors/recipes, not CLI argument strings. Document operation-specific options,
context selection, failures and cancellation. An implementation in an uncalled
module or an advertised but unhandled capability does not satisfy integration.

Preserve editor empty-pattern search, open loose documents and discovery of
unimported generated standard APIs as an explicit `editor_catalog` policy.
CLI find and rich extension find use `checked_context` by default: nonempty text,
owned declarations and eligible imports/exports. The shared collector handles
both policies; catalogue entries outside checked scope have no portable reference
and cannot establish uniqueness or mutation eligibility. Selecting such an entry
requires analysis in its owning context. No second resolver is introduced.
Standard LSP collections may project proven
facts but cannot claim coverage fields their protocol cannot express. Keep rich
coverage in the extension response and never treat a clipped standard list as
proof of uniqueness or mutation eligibility. Analysis failure/cancellation must
not become an apparently successful empty answer. Call Hierarchy shows proven
direct edges only; it does not fabricate dynamic edges or include callbacks as
calls. Grouping incoming/outgoing sites must preserve every retained call span.

## Refactoring and publication

Use one core recipe, eligibility check, edit generator and overlay validator
per operation. CLI plan files wrap its checked result; they are not required for
ordinary LSP operations. LSP prepare/revalidate uses synchronized overlays and
returns the same blockers and semantic edits when inputs/policies are equal.
Retain all relevant base hashes and open-document versions. A changed source,
dependency, context or relevant version invalidates the prepared result.

The CLI rechecks saved inputs and uses its coordinator publisher. LSP rechecks
current synchronized inputs before producing a `WorkspaceEdit`, with document
versions for open files. The editor applies edits through its normal workspace
mechanism and preserves unsaved state/undo. It does not route dirty buffers
through the CLI publisher or save them implicitly. Recheck closed-file hashes
before delivery. Extract/import CodeActions carry a command and snapshot-bound
recipe handle, not a cached edit. The registered execute-command handler
revalidates the full read set and requests `workspace/applyEdit`; it reports a
client rejection without claiming success. Add-parameter likewise revalidates
at invocation. An unavailable required client capability disables that route.
Standard rename validates in the final rename request, not only prepare-rename.

Document versions protect edited open files. Standard `WorkspaceEdit` cannot
atomically guard unedited dependencies or closed-file hashes after delivery.
Report stale state known before delivery; document the remaining client/writer
race instead of promising an atomic multi-file transaction or that every stale
dependency will be detected at application. Do not insert dummy edits into
unedited dependencies to simulate such a guarantee.

The VS Code inspect view and add-parameter command use `LanguageClientController`.
The latter accepts typed arguments for automation and asks for missing recipe
values during interactive use; it adds no extra approval gate. Existing extract
and import actions remain usable through their normal code-action interface.
Show relevant source, omission and blocker information without exposing transport
or hashing internals as mandatory user steps.

## Primary paths and regression owners

Repository paths below are affected implementation targets, not edits made by
this planning change. Individual work files name their feature-specific paths.

| Owner | Modify/create paths |
| --- | --- |
| Common core | `crates/fpas-language-service/src/analysis/`, `src/document/`, `src/navigation/source_query/`, `src/refactoring/`; `crates/fpas-project/src/source_inputs.rs` and its inventory module |
| Shared build identity | New `crates/fpas-language-service/build.rs`, `build/analysis_identity.rs`, `src/analysis/identity.rs`; modify that crate's `Cargo.toml` and module exports |
| LSP extension and refactoring-command registration | `crates/fpas-lsp/src/lib.rs`, `src/capabilities.rs`, `src/server/mod.rs`, `src/server/backend.rs`; `src/source_tools_requests.rs`, `src/server/source_tools.rs`, `src/refactor_requests.rs` and its modules, `src/server/refactoring.rs` |
| Existing LSP consumers | `crates/fpas-lsp/src/navigation_requests.rs`, `src/semantic_tools_requests.rs`, `src/navigation/`, `src/server/navigation.rs`, `src/server/semantic_tools.rs`, `src/documents/` |
| LSP Call Hierarchy | New `crates/fpas-lsp/src/call_hierarchy.rs`, `src/server/call_hierarchy.rs`; module/trait wiring in `src/lib.rs`, `src/server/mod.rs`, `src/server/backend.rs`, `src/capabilities.rs` |
| Editor connections | `editors/vscode/src/languageClient.ts`, `src/extension.ts`, `package.json`; new `src/sourceTools.ts` for registration and `src/sourceTools/` (`inspect.ts`, `refactor.ts`) for the respective commands |
| Core proof | Existing language-service `navigation_edits`, `analysis_state` and `project_context` targets |
| LSP proof | Existing `workspace`, `editing`, `analysis`, `protocol` targets; new `tests/source_tools.rs` and `tests/call_hierarchy.rs` in `tests/suites/workspace.rs`, new `tests/refactoring_plans.rs` in `tests/suites/editing.rs` |
| Editor proof | Existing `editors/vscode/test/refactoring.ts`, `test/extension.test.ts`, `test/language_client_lifecycle.ts`; `test/source_tools.ts` and `test/add_parameter.ts` registered in the extension suite |

Abbreviated paths in a row are relative to its named crate or editor directory.
Keep files focused; register new modules and handlers rather than relying on
automatic discovery. Retain the existing protocol ordering and cancellation
mechanisms when adding custom methods.

## Acceptance for every delivering work package

Core semantic tests own resolution/coverage/edit correctness. CLI tests own
argv, process, JSON, plan files and publication. LSP transcript tests own handler
registration, capabilities, URI/UTF-16 conversion, versions, cancellation and
edit projection. Editor tests own command registration, request dispatch, visible
results and actual buffer edits. Reuse existing fixtures; do not repeat the full
semantic matrix independently at every transport.

For each delivered operation, compare core/CLI/LSP semantic facts for the same
saved context, including identity, exact spans, blockers and stated coverage.
Also test dirty multi-file overlays, supplementary Unicode characters, stale
versions, source changes, incomplete analysis and cancellation. A shared-core
regression must have evidence that both real adapter routes observe the repair.
For cached code actions, change only an unedited dependency before command
execution and verify rejection. Exercise missing apply capabilities and client
apply rejection separately from successful source validation.
Run a real editor inspect/add-parameter workflow, including an unsaved source,
before their work packages are complete.

Compile Rust tests separately, then execute applicable targets:

```text
cargo test --locked -p fpas-lsp --test workspace --test editing --test analysis --test protocol --no-run
cargo test --locked -p fpas-lsp --test workspace --test editing --test analysis --test protocol
```

Use existing core/CLI commands from AP56.2. For affected editor connections run
`npm run compile` and `npm test` in `editors/vscode`; use the actual existing
suite registration and retain its environment requirements. Update the editor
handbook, refactoring/source-query handbook, extension README and declared
capabilities with delivery. No work-package checkbox closes with only its CLI
or core implementation complete.
