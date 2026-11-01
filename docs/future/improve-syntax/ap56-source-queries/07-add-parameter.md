# AP56.7: Add checked parameter insertion

Package: [AP56](README.md). Status: not started; depends on AP56.5 and AP56.6.
Uses Q85, Q86, Q88, Q89 and Q90. No FPAS grammar change is required.

## Contract and scope

Add `fpas refactor add-parameter` to AP56.6's existing preview/plan/apply flow.
The initial operation appends one parameter to a named source function or
procedure and appends an explicit argument to every proven supported call.
Proposed example:

```text
fpas refactor add-parameter --symbol App.Math.Clamp --context examples/pascal/units-basic/units-basic.fpasprj --name Offset --type integer --mode value --argument '0' --out parameter.plan.json
fpas refactor apply --plan parameter.plan.json
```

Require explicit name, existing FPAS type expression, mode (`value` or `var`)
and one argument expression. Parse each complete option value with the existing
grammar, requiring end-of-input; reject extra declarations, argument separators
and trailing tokens. Resolve the type in the declaration's scope. Parse the
argument as a value expression or, for `var`, an unmarked writable designator;
the generator supplies any named-argument label and `var` marker.
Interpret the argument in each caller's own scope and reject it if any supported
call fails validation. No inferred argument, import insertion, default-value
syntax or per-call policy file is added. Different per-call expressions,
removal/reordering of parameters,
method/constructor signature changes and arbitrary signature replacement are
outside this first operation.

The selection uses the same checked position/name/reference rules as rename.
Support ordinary direct calls in expression and statement form, including
recursive calls and calls through import aliases. Preserve positional calls as
positional; append the new named argument to named calls without reordering
existing argument expressions. Add the required `var` marker only when the
caller-supplied expression is checked writable storage under existing rules.

## Eligibility and completeness

Consume AP56.5's full typed relation inventory, not its bounded public result.
Before creating edits, enumerate the complete declaration-use and call inventory
inside the selected context. Block incomplete analysis, unresolved potentially
relevant calls, callable-value/callback uses, unsupported method/spawn sites and
generic cases excluded by the matrix below. Return exact blocking locations and
reasons; never edit a subset because its direct calls
are easy to find. A conservative refusal is preferable to guessed propagation.

The initial supported forms are fixed below; do not silently accept a form merely
because its edited version compiles. Generated APIs, source-less builtins and
out-of-root sources are not writable. Exported routines report that only selected consumers are
covered; no cross-repository safety claim is implied.

| Declaration or use | Initial decision |
| --- | --- |
| Named non-generic program/unit function or procedure | Supported, including exported routines within the reported consumer scope. |
| Named non-generic nested routine outside generic bodies | Supported with checked lexical bindings and unchanged captures. |
| Ordinary direct expression/statement call, including recursion and import aliases | Supported with exact declaration and argument/parameter mappings. Preserve named or positional form. |
| Direct call in a non-generic record-field default | Supported once at its physical source span, using the record declaration's environment, not each construction site. |
| Generic routine declaration, generic record default or an affected call inside a generic body | Unsupported in AP56.7; do not infer specialization coverage. Concrete applied generic parameter types remain ordinary type expressions. |
| Record/static method, fluent invocation, constructor/conversion, task spawn | Unsupported declaration or use; report the exact blocker. |
| Callable-value/callback use or unresolved potentially relevant invocation | Blocks the entire plan, even if every visible direct call is editable. |
| Compiler-bound implicit use, such as a resource release operation | Blocks the plan; the source-call query's exclusion of implicit calls does not make this declaration use safe to omit. |

Attach owning metadata and regression cases to this matrix during implementation.
Resolve missing internal links at their current owner without changing syntax.

## Checked edits and application

Use snapshot-owned AST/retained expression spans and original tokens for exact
parameter-list and call-list insertions. Preserve comments, contract clauses,
line endings and all untouched bytes.
An empty list and a semicolon-separated existing parameter list need distinct
insertion handling. Existing AST spans and lexer tokens should suffice; add a
missing retained delimiter span only if a concrete source case requires it.

Check identifier/type visibility, duplicate names, var storage and named-argument
labels, including collisions with the storage roots of existing `var` arguments.
Reanalyze every affected project/selected consumer against all overlays.
Compare unaffected bindings and call targets before/after so a new parameter
cannot silently shadow a previously used outer/global name while still compiling.
Map unchanged declaration/use spans across edits; raw before/after offsets alone
are not comparable semantic identities.
Preserve evaluation order/count of all existing arguments and receiver loads.
Evaluate the new argument after the existing written arguments, once on each
execution that reaches it. Its effects, failure or `try` propagation may prevent
the call; disclose this rather than claiming behavioral equivalence. Check a
recursive argument against the before snapshot too, so a newly added parameter
cannot accidentally supply itself through name capture. New calls to the changed
routine inside the argument are unsupported: reject them before edit generation
instead of repeatedly expanding the recipe.

Produce AP56.6's deterministic plan with the exact recipe and complete edits.
Refusals and preview never alter source. CLI apply regenerates and validates the
same edits against the unchanged inventory, then uses the shared publisher and
recovery contract. Reapplication is already-applied or stale, never a second
parameter insertion. Introduce no separate mutation or rollback mechanism.

## Required LSP/editor connection

Add the parameter recipe to `fpas/refactor` prepare/revalidate and a registered
VS Code add-parameter command. The command accepts explicit typed arguments for
automation and collects missing values in interactive use. It calls the LSP
adapter, shows blockers or the prepared change, revalidates current snapshots,
and applies a versioned `WorkspaceEdit` through the editor. Use the same core
eligibility, argument checks, binding validation and edit generation as CLI.
Include dirty callers/dependencies, preserve unsaved state/undo and add no implicit
save or CLI subprocess. Follow the
[adapter contract](adapter-contract.md#refactoring-and-publication).

## Affected files

| Action | Repository path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-language-service/src/refactoring/add_parameter.rs` | Eligibility, checked insertion edits and operation invariants |
| Modify | `crates/fpas-language-service/src/refactoring/` (`mod.rs`, `plan.rs`, `validation.rs`) | Recipe dispatch and complete overlay/binding validation |
| Reuse/modify as needed | `crates/fpas-language-service/src/navigation/source_query/calls.rs`, `crates/fpas-language-service/src/navigation/checked_bindings.rs` | Full relation inventory and proven type/binding identities |
| Conditional modification | `crates/fpas-parser/src/ast/routines.rs`, `crates/fpas-parser/src/parser/decl/parameters.rs`, existing call metadata owners named in AP56.5 | Retain a demonstrated missing source span or checked link; preserve grammar |
| Modify | `crates/fpas-cli/src/cli_input/refactor.rs`, `crates/fpas-cli/src/cli_refactor/` (`mod.rs`, `plan.rs`), shared worker/protocol/report modules | Arguments, typed recipe and bounded blocker/edit rendering |
| Create/register | `crates/fpas-language-service/tests/refactoring_add_parameter.rs`, `crates/fpas-language-service/tests/suites/navigation_edits.rs` | Parameter/call semantics, bindings and refusal boundaries |
| Extend | `crates/fpas-cli/tests/refactor.rs` | End-to-end preview, exact apply, stale/repeat outcomes and no writes on refusal |
| Extend | `crates/fpas-lsp/src/source_tools_requests.rs`, `src/server/source_tools.rs`, `tests/refactoring_plans.rs` under that crate | Parameter recipe preparation/revalidation, versioned edits and matching blockers |
| Create/modify | Editor connection paths in `adapter-contract.md`, including `editors/vscode/src/sourceTools/refactor.ts`, `test/source_tools.ts` | Registered add-parameter command with typed recipe input and a real editor application workflow |

Keep eligibility and edit construction focused; split files by those concerns
if they approach the repository's size guideline. Reuse the existing edit model,
parser, sema and publisher rather than adding a string-based transformation engine.

## Documentation and verification

Extend refactoring help/handbook, CLI reference and agent tools reference with
the precise supported forms, argument policy and blockers. Planned unsupported
operations stay out of installed help. Deliver and document the LSP extension,
editor command and shared semantic contract in the same work package.

Cover empty/existing parameter lists, zero/multiple/recursive call sites, named
and positional arguments, imported aliases, the fixed nested/generic boundaries,
record defaults with several construction sites, name shadowing, invisible types,
colliding var storage roots, side-effect ordering, contracts, comments,
complete-fragment parsing, self-referential new arguments and Unicode/line endings.
Test callable values, callbacks, dynamic calls, implicit resource uses, spawns,
methods, external scope and broken sources as explicit blockers. Rejected,
clipped or stale plans publish no edits; repeat apply never duplicates a change.

Use the existing `navigation_edits` and CLI `tools` targets, registering new
modules once. Compile separately, then execute the semantic and CLI refactor
cases as in [AP56.6](06-refactor.md). Complete all package-wide Rust checks and
applicable formatter/example/help/document checks before marking AP56 complete.
Apply [shared constraints](../shared-constraints.md). Verify deterministic edits,
documented blockers and complete validation on fixed source fixtures; no LLM task
benchmark or model-dependent acceptance gate is required.
Run LSP editing/protocol and editor checks from the adapter contract. Compare
saved-context edits/blockers with CLI, then cover unsaved declarations/callers,
concurrent version changes, Unicode conversion and an actual command-to-buffer
application without saving. AP56.7 cannot close with only CLI parameter insertion.
