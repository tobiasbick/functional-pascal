# AP56.6: Expose checked refactoring plans and apply

Package: [AP56](README.md). Status: not started; depends on AP56.2 and AP56.3.
Uses Q85, Q86 and Q89. Direct call queries are not a prerequisite for rename,
extract-variable or organize-imports.

## Contract and scope

Expose existing checked transformations through one non-interactive
`fpas refactor` family. Preparation returns a plan without changing source.
Only CLI `fpas refactor apply --plan <file>` publishes changes directly to disk.
Both steps can run autonomously within an assigned editing task; no interactive
prompt or additional user approval is required.
Proposed examples:

```text
fpas refactor rename --symbol App.Math.Clamp --name Bound --context examples/pascal/units-basic/units-basic.fpasprj --out rename.plan.json
fpas refactor extract-variable --range examples/pascal/units-basic/src/math_utils.fpas:24:10:24:24 --context examples/pascal/units-basic/units-basic.fpasprj --out extract.plan.json
fpas refactor organize-imports examples/pascal/units-basic/src/math_utils.fpas --context examples/pascal/units-basic/units-basic.fpasprj --out imports.plan.json
fpas refactor apply --plan rename.plan.json
```

Rename accepts the shared position/name/reference selectors. Organize-imports
accepts one source file. Extract-variable accepts `--range
<file>:<start-line>:<start-column>:<end-line>:<end-column>` with one-based scalar,
half-open positions. Parse coordinates from the right, preserving drive colons.
Require exactly the selector appropriate to the action. Extract-variable keeps
the existing deterministic, collision-free generated binding name and exposes
that name in its plan; a separate naming option is unnecessary initially.

Migrate `query rename` to `refactor rename` and remove the old command, immediate
write mode and `--dry-run` switch. There is one canonical plan/apply flow, with
no compatibility alias. AP56.2 first unifies the existing rename infrastructure;
this work package changes its command family and publication trigger together
with help, tests and handbook examples.

Use the existing language-service rename and `src/refactoring/` implementations.
Preserve extraction's evaluation/order/frequency restrictions and import
organization's comments, aliases and initializer-effect checks. Generalize their
snapshot-bound edit results and overlay validation where needed; do not create
another resolver or reimplement the transformations in the CLI.
Replace the current optional-edit result at the core boundary with distinct
prepared, no-change, unsupported and incomplete results, including reasons.
An absent editor action currently conflates these cases. For example, imports
already in order are no-change; comments that prevent safe organization are an
unsupported case. CLI and LSP must preserve that distinction.

## Plan artifact and output

Use the shared schema-2 JSON envelope with a tagged refactoring operation.
`result.plan` contains a versioned, deterministic recipe, selected target/range,
complete before/expected-after inventory, exact ordered edits, original text,
replacement bytes, checked scope/limitations and a canonical content digest.
No timestamps or random identifiers participate in plan identity.

Without `--out`, return the complete preview/plan on stdout. With `--out`, also
atomically save that same UTF-8 envelope, so `apply --plan` reads one documented
form. Reject an output path aliasing any source, manifest or analysis input.
The destination must not alter recorded source membership. Do not overwrite a
different existing plan; identical bytes are a harmless repeat. Neither
preparation nor an artifact-output failure changes FPAS source.
Apply public JSON and internal preparation budgets before writing an artifact;
never truncate edit bytes into an applicable plan.
An eligible operation with nothing to change returns `ok` with an empty plan
and an explicit no-change reason; it does not invent edits to make progress.

A plan is untrusted input, not filesystem authority or proof of correctness.
Its digest detects changed content; it is not a signature. Reject unsupported
versions/actions, malformed/overlapping edits, outside-root or generated-API
targets and invalid path identities. Reconstruct the manifest context and
library selection; a supplied list of files cannot expand write scope.
Bound plan-file reads and decoding by the 8 MiB public ceiling before allocating
the internal preparation plan. Resolve identity paths against the recorded
context/path base, never against the apply invocation's working directory.

## CLI apply and recovery

Validate the plan version, structure, digest and context first, then reload the
complete frozen inventory. Compare inventory states before resolving the old
target: a successful rename or insertion normally invalidates its old reference.

- If the before inventory matches, re-resolve the recipe, regenerate its exact
  checked edits and compare them with the plan. Validate all affected projects
  and selected consumers against overlays before publication.
- If only the full expected-after inventory matches, return `ok` with
  `result.status: already_applied` and no writes. This reports matching saved
  state, not authenticated execution history. Do not resolve the old reference
  against the new source or run the transformation again.
- If neither matches, return `stale_plan` without writes. A partial match or an
  unrelated input/scope change never authorizes finishing a guessed remainder.

An eligible empty plan has equal before/after inventories; return `no_change`
after recipe validation. Malformed plans remain errors in every state.
Reapplication must never duplicate an extraction or insert another parameter.

Reuse AP56.2's bounded preparation and coordinator publisher. Verify every
analysis input and stage every replacement before publishing. Once replacement
begins, finish publication or ordinary recovery rather than killing the writer
at the read deadline. Report unchanged/replaced/restored/unresolved paths and
original/recovery errors. Preserve permissions, untouched bytes and concurrent
newer text; retain the documented limits on crashes and multi-file atomicity.
If final stdout delivery fails after changes, stderr identifies that report
delivery failed; callers must inspect saved files before retrying.

## Required LSP connection

Route standard rename, extract-variable and organize-imports through the same
core recipes, eligibility checks and whole-context overlay validation as CLI.
Expose prepare/revalidate through `fpas/refactor`; convert checked core edits to
versioned `WorkspaceEdit` values. CLI plan files and disk publication are not
part of the LSP workflow. Preserve unsaved buffers and normal editor undo.
Revalidate affected inputs and versions before returning/applying edits; stale
or unsupported transformations receive the same semantic blockers as CLI.
Offer extract/import actions as command-backed actions holding a snapshot-bound
recipe handle, without a cached `WorkspaceEdit`. On command execution, revalidate
the complete read set before requesting editor application. Register the command
and capability; an unavailable apply route disables the action. Standard rename
revalidates in its final rename request. The adapter contract defines the
remaining race between returning an edit and the client's application.
Follow the [adapter contract](adapter-contract.md#refactoring-and-publication).

## Affected files

| Action | Repository path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-cli/src/cli_input/refactor.rs`, `crates/fpas-cli/src/cli_refactor/` (`mod.rs`, `plan.rs`, `apply.rs`) | Canonical arguments, plan artifact handling and apply coordination |
| Modify | `crates/fpas-cli/src/cli_input/` (`mod.rs`, `mode.rs`, `types.rs`, `help.rs`), `crates/fpas-cli/src/cli_run.rs`, `crates/fpas-cli/src/main.rs` | Dispatch/help and early structured errors |
| Modify | `crates/fpas-cli/src/cli_input/query.rs`, `crates/fpas-cli/src/cli_query.rs`, `crates/fpas-cli/src/cli_query/` (`request.rs`, `worker.rs`, `protocol.rs`, `report.rs`, `rename.rs`, `publication.rs`) | Reuse one transport/publisher and remove the query-rename route |
| Modify | `crates/fpas-language-service/src/refactoring/` (`mod.rs`, `plan.rs`, `validation.rs`) | Extend AP56.2's shared edit model/validation with typed recipes and reusable plans |
| Reuse/modify as needed | `crates/fpas-language-service/src/refactoring/` (`extract.rs`, `imports.rs`), `crates/fpas-language-service/src/navigation/rename.rs`, `crates/fpas-language-service/src/lib.rs` | Existing checked actions and shared exports |
| Create/register | `crates/fpas-language-service/tests/refactoring_plans.rs`, `crates/fpas-language-service/tests/suites/navigation_edits.rs` | Plan invariants and cross-project validation |
| Create/register | `crates/fpas-cli/tests/refactor.rs`, `crates/fpas-cli/tests/tools/main.rs` | Real CLI preview/artifact/apply lifecycle |
| Migrate | `crates/fpas-cli/src/main_tests/query/rename.rs`, `crates/fpas-cli/src/main_tests/query.rs`, `crates/fpas-cli/src/main_tests/input/help.rs` | Replace old command expectations; keep primary mutation-safety coverage |
| Modify/extend | `crates/fpas-lsp/src/navigation_requests.rs`, `src/semantic_tools_requests.rs`, `src/server/semantic_tools.rs`, `src/source_tools_requests.rs`, `src/server/source_tools.rs`, `src/navigation/rename.rs` | Shared prepare/revalidate operations, code actions, extension handlers and versioned edit projection |
| Create/extend | `crates/fpas-lsp/tests/refactoring_plans.rs`, `tests/suites/editing.rs`, `tests/refactoring.rs`, `tests/references_rename.rs`; `editors/vscode/test/refactoring.ts` | Both LSP routes, common blockers and actual unsaved buffer edits |

Keep transport, public wire rendering and source publication shared rather than
forking a second subsystem under `cli_refactor`. Move common modules if that
improves ownership; record their exact destinations before implementation.

## Documentation and verification

Create `docs/pascal/tools/refactoring.md` on delivery. Update the source-query
handbook's rename section, tools index, CLI reference, agent tools reference and
focused help. Search tracked scripts, skills, editor adapters and examples for
old CLI usage; migrate actual consumers. Migrate existing LSP refactoring routes
to the core validator and update editor/extension documentation and capabilities.

Reuse existing `tests/refactoring.rs`, rename-safety and publication tests.
Cover exact preview/application, deterministic plans, artifact path collisions,
altered/oversized plans, stale unedited inputs, aliases, root/generated-source
guards, overlay failures, staging/replacement/recovery errors and stdout failure.
Test repeat after rename, extraction and import removal, partial application,
equal before/after no-change plans, changed working directories and bounded plan
decoding without assuming crash-atomic multi-file edits. Test unsupported versus
no-change results, including comments in imports and invalid extraction ranges.
Exercise extraction and import organization through the CLI while keeping their
semantic proof at the existing language-service owner.

Register the new CLI module in `tools` and semantic cases in `navigation_edits`.
Compile test targets before execution, then run those targets and CLI binary
rename/publication/help tests. Follow [AP56.2](02-foundation.md)'s commands and
the package build rules; add the `refactor::` filter for the CLI `tools` target.
Validate help/examples/links and `git diff --check`. Apply
[shared constraints](../shared-constraints.md).
Include the adapter contract's LSP editing/protocol and editor suites. Compare
core edit sets/blockers for equal saved inputs, then prove that dirty buffers
use overlay analysis, stale versions reject edits, and the real editor applies
the versioned result without an implicit save. Both adapters are required.
Request an extract/import action, change only an unedited dependency, then invoke
the cached action: it must reject the stale handle and apply no edit.
