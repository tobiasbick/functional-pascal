# AP56.3: Add checked find and declaration selection

Package: [AP56](README.md). Status: not started; depends on AP56.2.
Uses the recorded Q85 contract.

## Prerequisites

[AP56.2](02-foundation.md) delivers the common context, snapshot, reference,
coverage and report contract on `ap56`. Existing workspace-symbol search supplies
candidate metadata but does not establish checked query coverage.

## Contract and scope

Implement the detail plan's [name discovery and selection](../../agent-source-queries.md#selectors-and-context).
These are proposed forms until delivered:

```text
fpas query find Clamp --context examples/pascal/units-basic/units-basic.fpasprj
fpas query definition --symbol App.Math.Clamp --context examples/pascal/units-basic/units-basic.fpasprj
fpas query type --target-ref clamp.target.json
```

`find` requires nonempty search text and an explicit manifest. Name selectors
also require a manifest; a target-reference file supplies its recorded context,
with any explicit override required to agree. Accept exact short/qualified names
for selection. Partial matching belongs only to find. Preserve positional
selection for expressions, aliases, locals and shadowed names.

Return checked declaration identity, kind, callable detail when available,
documentation, source origin, visibility and owning project. Syntax-only
candidates remain labeled and cannot prove uniqueness or absence. Reuse the
existing documentation extractor; absent documentation stays absent.

Search owned project declarations, including locals, and eligible dependency
exports. Filter by project import/export context; hashing an analysis input does
not make its private declarations searchable. Select a dependency's own context
for declarations private to it. Include eligible imported standard APIs
and builtins without indexing all Std implementation bodies by default.

Rank exact short/qualified matches first, then the detail plan's partial-match
classes and deterministic tie-breakers. Retain a bounded top-N during traversal;
do not collect an unbounded vector before truncation. Defaults are 100 results,
maximum 10,000. Report omitted candidates and known/unknown totals.

A name resolves only after complete eligible discovery establishes exactly one
checked declaration. Multiple candidates yield `ambiguous_target`; interrupted
or clipped discovery cannot silently select its first match. A retained exact
target reference follows Q85's separate scope-verification rule. `type --symbol`
returns a declaration's value type, including a routine's callable type; it does
not invent a call result. Non-value declarations receive the specified outcome.
Source-less builtins retain their known identity without a fabricated source
span or reusable reference.

Enable these selectors for existing query operations, including rename, only
with the complete checked selector path. Preserve all AP56.2 mutation checks.
Call relationships and refactoring delivery belong to AP56.5 through AP56.7.

## Required LSP connection

Migrate `workspace/symbol` to the shared bounded candidate collector. Keep
empty searches, open loose documents and unimported standard API discovery under
the adapter contract's `editor_catalog` policy. CLI and rich extension find use
`checked_context` by default. Both routes share collection, ranking and current
overlays, but catalogue-only entries cannot issue checked portable references.
Add `find` and declaration selection to `fpas/sourceQuery` with rich coverage and
ambiguity. Compare adapter results only with the same explicitly selected policy.
Standard symbol lists are projections; they never authorize uniqueness or edits.
Follow the [adapter contract](adapter-contract.md#required-operation-mapping).

## Affected files and implementation owners

| Action | Repository path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-language-service/src/navigation/source_query/find.rs`, `selection.rs` in that directory | Bounded discovery, semantic candidate identity and exact selection |
| Modify | `crates/fpas-language-service/src/navigation/source_query/` (`mod.rs`, `model.rs`) | Integrate the new operations with shared scope/results |
| Modify as needed | `crates/fpas-language-service/src/symbols/index.rs`, `crates/fpas-language-service/src/navigation/workspace_symbols.rs` | Reuse matching/index traversal while preserving existing editor behavior |
| Modify | `crates/fpas-cli/src/cli_input/query.rs`, `crates/fpas-cli/src/cli_query/` (`worker.rs`, `report.rs`), `crates/fpas-cli/src/main_tests/input/help.rs` | Enable find/selectors and render their typed results/help |
| Create/register | `crates/fpas-language-service/tests/source_query_find.rs`, `crates/fpas-language-service/tests/suites/navigation_edits.rs` | Own semantic scope, name ambiguity and ranking proof |
| Create/register | `crates/fpas-cli/tests/source_queries/find.rs`, `crates/fpas-cli/tests/source_queries/mod.rs` | Own process JSON, selector errors, reference reuse and bounds |
| Modify/extend | `crates/fpas-lsp/src/navigation_requests.rs`, `src/navigation/workspace_symbols.rs`, `src/source_tools_requests.rs`, `src/server/source_tools.rs`; `tests/source_tools.rs`, `tests/workspace_navigation.rs` under that crate | Connect standard/rich search, projections and overlay/parity regressions |

## Migration and documentation

Add focused `find --help` with examples and canonical selector help for existing
operations. Update the source-query handbook, CLI reference and agent tools
reference. Preserve current positional expression semantics and editor/LSP
workspace-symbol consumers; migrate internal Rust types in the same change.

## Acceptance and verification

Use real temporary projects/workspaces for positive, negative and boundary cases:
case variants; same short name; same qualified name across project owners;
aliases and shadowing; nested/generic declarations; exported/private dependency
symbols; excluded files; generated APIs and source-less builtins. Declaration
discovery must not turn visibility into permission to call a private symbol.

Verify exact-before-partial ranking and deterministic output when source discovery
order changes. Small limits retain a bounded result with explicit omissions.
Broken sources, missing metadata and cancelled discovery never yield a complete
empty result or a unique short-name selection. Reference reuse succeeds unchanged
and rejects source, manifest, membership, library and fingerprint changes.
No program executes during discovery.

Extend existing navigation/coverage fixtures where they already own a contract.
Register new modules once in the existing targets. Compile then execute:

```text
cargo test --locked -p fpas-language-service --test navigation_edits --no-run
cargo test --locked -p fpas-cli --bin fpas --test tools --no-run
cargo test --locked -p fpas-language-service --test navigation_edits
cargo test --locked -p fpas-cli --test tools source_queries::
cargo test --locked -p fpas-cli --bin fpas query::
cargo test --locked -p fpas-cli --bin fpas input::help::
```

Check focused help, relative documentation links and `git diff --check`. Apply
the package's build/resource rules and [shared constraints](../shared-constraints.md).
Complete AP56.3 only with find and enabled selectors documented and verified.
Include LSP workspace/extension tests from the adapter contract. Compare equal
saved contexts with CLI results, then exercise dirty dependent files, empty
editor search, unimported API discovery, loose documents, ambiguity, clipping,
cancellation and supplementary Unicode. Catalogue entries must not widen checked
name-selection or mutation scope.
Both actual adapter routes are required for this work package's completion.
