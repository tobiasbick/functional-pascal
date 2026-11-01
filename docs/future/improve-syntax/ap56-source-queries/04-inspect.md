# AP56.4: Add bounded inspect

Package: [AP56](README.md). Status: complete; depends on AP56.3.

The shared core, isolated CLI worker and native LSP expose bounded inspection.
The VS Code command/view is registered and its real Extension Host workflow
passes with dirty dependencies, read-only rendering and stale-view rejection.
The core, CLI and LSP inspection cases pass within the complete workspace checks.

## Prerequisites

- [AP56.3](03-find.md) supplies checked position/name/reference selection.
- Q87 records inspection scope, fixed budgets and omission meanings.
- Reuse snapshot-owned AST/retained allocations and checked declaration/binding metadata;
  existing `DocumentSymbol.type_name` strings alone do not prove type identity.

## Contract and scope

Deliver the detail plan's [bounded inspection](../../agent-source-queries.md#phase-2-bounded-inspect-context).
Implemented selectors:

```text
fpas query inspect --symbol App.Math.Clamp --context examples/pascal/units-basic/units-basic.fpasprj
fpas query inspect --target-ref clamp.target.json
fpas query inspect examples/pascal/units-basic/src/math_utils.fpas:5:17 --context examples/pascal/units-basic/units-basic.fpasprj
```

Return sections in order: signature/documentation, implementation when physically
available, then directly relevant nominal types from the signature, receiver and
explicit local annotations. Each section includes its reason, semantic owner,
source range/hash, origin, analysis state and excerpt/truncation information.
Keep declaration and implementation locations distinct when necessary. Preserve
contract clauses when presenting a routine's source declaration.

Use resolved types and snapshot AST spans for declaration and body lookup.
Expand structural wrappers to their directly named type arguments and deduplicate
semantic identity. Keep one-hop scope: no recursive member-type, callee, import
or method-body expansion. Missing bindings produce unavailable context rather
than name-based guesses. Use current metadata before extending sema.
Include a directly named type or source alias declaration once, in signature,
receiver, then local-annotation priority and source order.

Distinguish FPAS source implementation from generated intrinsic declaration
files. Generated `lib/api/Std` bodies such as a panic stub are never returned as
native implementation. A physical declaration location may still be useful.
Use the standard-library context's source-origin metadata, not a search for stub
body text. Source-less builtins and other unavailable bodies have explicit reasons.

Apply Q87's fixed budgets and the shared input/JSON ceilings during construction.
For large types or bodies, return labeled exact excerpts with full locations,
sizes and known omission counts. Preserve Unicode boundaries and source line
endings. Disjoint excerpts must not appear to be one continuous declaration.
Text clipping and deliberate one-hop context limits remain distinct from
incomplete semantic analysis; identity/edit fields are never clipped.

## Required LSP/editor connection

Expose the same typed inspection through `fpas/sourceQuery` and a VS Code
inspect command/read-only view. Render signature, implementation, supporting
types and omissions from those returned sections, including unsaved dependencies.
Reuse shared declaration/type facts in existing hover adapters without imposing
full inspection cost on hover. Register and advertise the method/command; do not
spawn the CLI or read disk in place of dirty buffers. Follow the
[adapter contract](adapter-contract.md#required-operation-mapping).

## Affected files and implementation owners

| Action | Repository path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-language-service/src/navigation/source_query/inspect.rs` | Checked declaration/body lookup and bounded supporting context |
| Modify | `crates/fpas-language-service/src/navigation/source_query/` (`mod.rs`, `model.rs`) | Integrate typed sections and omission reasons |
| Reuse/modify as needed | `crates/fpas-language-service/src/` (`documentation.rs`, `symbols/document.rs`, `analysis/document.rs`, `navigation/checked_bindings.rs`, `workspace/standard_library.rs`) | Existing documentation, spans, checked type identities and generated-source provenance |
| Conditional modification | `crates/fpas-sema/src/check/context.rs` and the owning binding-resolution code | Retain a demonstrated missing type link on its owning snapshot AST/retained allocation |
| Modify | `crates/fpas-cli/src/cli_input/query.rs`, `crates/fpas-cli/src/cli_query/` (`worker.rs`, `report.rs`), `crates/fpas-cli/src/main_tests/input/help.rs` | Focused help, shared dispatch and bounded section rendering |
| Create/register | `crates/fpas-language-service/tests/source_query_inspect.rs`, `crates/fpas-language-service/tests/suites/navigation_edits.rs` | Own one-hop selection, body provenance and supporting-type semantics |
| Create/register | `crates/fpas-cli/tests/source_queries/inspect.rs`, `crates/fpas-cli/tests/source_queries/mod.rs` | Own JSON sections, exact excerpt bytes and public limits |
| Modify/extend | `crates/fpas-lsp/src/source_tools_requests.rs`, `src/server/source_tools.rs`, `src/navigation_requests.rs`; `tests/source_tools.rs` under that crate | LSP inspection, shared hover facts and overlay/staleness conversion tests |
| Create/modify | Editor connection paths in `adapter-contract.md`, including `editors/vscode/src/sourceTools.ts`, `src/sourceTools/inspect.ts`, `test/source_tools.ts` | Registered inspect command/view using the language client, with a real editor workflow |

No parser traversal framework or semantic display-string parser is a prerequisite.
If new sema metadata is needed, identify its exact owner and distinct regression
before changing it. Native Rust implementation lookup is outside AP56.

## Migration and documentation

Add `inspect --help` with position, name and reference examples. Update the
source-query handbook, CLI reference and agent tools reference. Explain fixed
budgets and unavailable implementation separately from no declaration. Keep
later call queries out of installed help. No grammar or VM change is required.

## Acceptance and verification

Cover functions/procedures, methods/receivers, generic and nested declarations,
records/enums, aliases and distinct types. Assert that selected supporting types
have compiler-proven identities, including shadowed/imported names, and that
unrelated/transitive types stay outside the request. Recursive types terminate.

Inspect one generated standard API declaration and one source-backed standard
routine. Only the latter may expose a real FPAS implementation body. Cover
source-less builtins, missing metadata, private dependency visibility, absent
documentation, stale references, Unicode/line endings and oversized context.
Every excerpt reproduces its exact source range; omissions and limits remain
explicit. The query executes no program and compiles no bytecode.

Compile and execute the existing `navigation_edits`, `analysis_state` and CLI
`tools` targets as in AP56.2, including the registered inspection cases. Run
the package's complete applicable Rust checks, affected help/document examples,
agent-reference checks when changed, relative links and `git diff --check`.
Use the [shared constraints](../shared-constraints.md).

Verify fixed declaration/context fixtures for correctness, omissions and the
documented output budgets, without model-based task comparisons. Complete this
work package's checkbox after delivery; call/refactoring work remains in AP56.
Completion also requires the adapter contract's LSP/editor checks: equal saved
contexts produce equivalent sections, dirty buffers are reflected, stale views
are refreshed or rejected explicitly, and the registered editor command works.
