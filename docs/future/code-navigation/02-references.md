# 02: N1 references and shared query foundation

Next implementation step. Deliver a usable `fpas query refs` under the
[shared contract](01-contract.md). The [overview](README.md) owns status.

## Existing owners to reuse

- [Navigation service](../../../crates/fpas-language-service/src/navigation/service.rs)
  exposes definitions and `references_with_cancellation`; its current result
  does not distinguish an unresolved target from a declaration with no uses.
- [Reference traversal](../../../crates/fpas-language-service/src/navigation/references.rs)
  filters candidate tokens by resolved declaration identity and retains the
  exact source snapshot for each returned span.
- [Workspace ownership](../../../crates/fpas-language-service/src/workspace/context.rs),
  [visibility](../../../crates/fpas-language-service/src/workspace/project.rs) and
  [analysis](../../../crates/fpas-language-service/src/analysis/project.rs)
  own source sets, member/import semantics and checked binding information.
  Existing editor recovery and implicit source discovery are not by themselves
  the strict scope/completeness contract of a query command.
- [CLI dispatch](../../../crates/fpas-cli/src/cli_run.rs),
  [input/help](../../../crates/fpas-cli/src/cli_input), and
  [API lookup](../../../crates/fpas-cli/src/cli_doc) provide nearby command patterns.
- [Batch hosting](../../../crates/fpas-cli/src/cli_batch) and
  [process containment](../../../crates/fpas-cli/src/process_tree) provide the real
  parent/worker, publication and timeout boundaries. Debugger-specific input,
  stop state and output capture remain debug-owned.

## Work

Add focused query input/dispatch/report modules (expected owner: a new
`crates/fpas-cli/src/cli_query/` directory). Do not grow `cli_run.rs` into a
second language service. Select JSON error output from raw query mode before
full argument parsing, as with D3. Preserve ordinary help/version behavior.

Expose a typed language-service query result containing selected target,
references, diagnostics and coverage. Existing editor methods can remain thin
adapters; do not infer completeness from an empty `Vec`. Extend shared ownership
and cancellation APIs only where the real query/editor callers need them.

Freeze the actual analysis input set and keep checked types/binding identities
for candidate documents where member inference requires them. Inspect the
reference-context rebuild: rebuilding raw navigation documents must not discard
inferred receiver or generic information and silently lose uses. Add support for
manifest-selected test programs without treating several programs as one unit.

Apply source/result limits before unbounded collection. Preserve original FP
codes and source attribution; use named query diagnostics for query-specific
misses/limits. Snapshot hashes and a final consistency check distinguish a useful
partial answer from a complete answer. Include the selected context in the result.

Use parent-owned isolated execution for the end-to-end timeout. No compiler,
FPAS bytecode or user-program launch. Reuse atomic/private IPC publication and
existing process-tree behavior; do not implement another platform kill strategy.

## Acceptance at the owning boundary

- Real CLI: default JSON, every envelope/exit class, strict options and duplicate
  handling, bad paths/coordinates, Windows-drive suffixes, whitespace in paths,
  and one stdout document with no raw stderr leakage.
- Unicode scalar input/output coordinates, CR/LF variants, BOM, exact UTF-8 spans,
  token-end/EOF selection and previews tied to the correct snapshot.
- Local/parameter shadowing, case-insensitive identifiers, cross-unit aliases,
  private/exported symbols, inferred record receivers, enum/record patterns,
  named actual/pattern labels, recursion, comments/strings and equal member names.
  Extend the existing language-service reference tests rather than duplicating
  their full matrix at every CLI layer.
- Unused declaration returns `ok`/0 with an empty use list and a known target;
  unresolved token, an explicitly unsupported source-less built-in and missing
  analysis remain distinct. Source-backed Std API declarations stay supported.
- Project, multi-member workspace, test project, dependencies and loose file;
  ambiguous ownership and an explicit out-of-scope source never widen the search.
- Recovered parse/type errors retain only justified matches with exit 3; unreadable
  or changed sources/manifests and truncated discovery cannot claim completeness.
- Limits bound reading, analysis/retention and the escaped document; first useful
  partial data and primary diagnostics survive clipping. Exercise high-escape
  argv separately from the source/IPC/result ceilings.
- Real blocked preparation, deadline/cancellation, worker death and private-file
  cleanup. Validate platform behavior where available; keep D3/test-runner
  lifecycle coverage at the shared owner after any extraction.

Document N1 as shipped only after these checks and independent review pass.
Keep N2/N3 help entries absent until their respective implementation lands.
Update the wish list, plan status and permanent CLI/tooling docs before step 03.
