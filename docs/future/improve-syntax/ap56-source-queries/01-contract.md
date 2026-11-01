# AP56.1: Specify the common contract and fixtures

Package: [AP56](README.md). Status: complete; see the [wire contract](wire-contract.md).
This work package turns recorded
technical choices Q85 through Q91 into precise schemas, encoding vectors and
worked examples. It requires no user decision or feature implementation.

## Prerequisites

Q84 selects the package scope and branch. The
[detail plan](../../agent-source-queries.md#planning-baseline) records the
code-backed baseline. Confirm those owning paths before implementation.

## Selected technical direction

### Q85: Schema, selection and snapshot identity

- Replace both schema-1 shapes with the detail plan's schema-2 envelope. Use
  `--context` with manifests for every explicit context, including rename.
  Preserve implicit positional discovery, help/version output and source ranges.
- Introduce typed domain requests/results in the language service and typed
  adapter projections in CLI and LSP. Both call the same semantic operations;
  neither resolves names independently. Preserve original attributed diagnostics.
- Define one bounded, operation-independent input scope per context/library.
  Include every participating source/manifest hash, selected membership,
  lexical-to-physical path binding and analysis-affecting option. Keep searched
  declarations separate from private sources needed only for analysis.
- Use SHA-256 over a versioned canonical encoding. Specify field order, UTF-8
  byte-length framing, sorted inventories and path normalization with fixed
  vectors. Reject unsupported path identities explicitly rather than hashing
  lossy display paths. Public display paths retain their existing conventions.
- Use a shared analysis-core build fingerprint, not either executable's hash.
  Specify the versioned source/dependency/build inputs and canonical encoding
  from the [adapter contract](adapter-contract.md#source-modes-identity-and-limits).
  AP56.2 generates this identity once for the shared crate. An unavailable
  fingerprint prevents portable reference issuance/reuse. Source/library hashes
  remain separate; compiler-artifact identity is unchanged.
- A source-backed target reference identifies its context, snapshot,
  owner/project, declaration kind, canonical file, name span and file hash.
  Reuse rechecks the complete inventory and identity. Changes return
  `stale_target`; no automatic retargeting. Source-less builtins remain selectable
  by position/name, with a null reference and an explicit reason; invent no file
  or span. A generated API declaration with a real source span may have a
  reference even though its implementation body is unavailable.
- Distinguish verified scope from returned-collection limits. A clipped find list
  may carry references for retained checked declarations only if the full scope
  was verified; it cannot establish uniqueness by a shortened name. Incomplete
  analysis or changed inputs produce no reusable references. Rename always
  performs its own complete validation before publication.

The public schema fixtures must cover each outcome, null/unavailable fields,
ambiguity candidates, per-collection coverage, target references and limits.
Specify encoding and fingerprint examples here before AP56.2 starts.

### Q86: Rename preparation and publication

- Let the isolated worker prepare a fully checked edit plan over frozen inputs.
  Return an owned inventory and exact edit bytes to the surviving coordinator;
  the thread-local `SourceInputs` guard itself cannot cross that boundary.
- Apply the deadline to preparation and pre-publication checks. Once replacement
  starts, the coordinator finishes publication or ordinary recovery. Process
  termination and concurrent writers remain outside atomic multi-file guarantees.
- Define a separate bounded internal plan transport with a 128 MiB total
  budget for retained original/replacement bytes, with bounded metadata as well.
  Keep the existing 8 MiB public JSON limit. Reject an oversized plan or public
  edit report before writing sources; never clip replacement text or target IDs.
- Report per-path states for unchanged, replaced, restored and unresolved files,
  including the original publication error and any recovery error. Preserve
  permissions and newer concurrent contents. Stderr is reserved for final report
  delivery failure, which can occur after source changes.

Specify publication-state fields, preparation limits and cancellation boundaries
in schema fixtures. Read-only queries never enter this publication path.

### Q87: Inspection scope and budgets

- Start with fixed budgets of 32 KiB aggregate source excerpts and 12 supporting
  types, in addition to the detail plan's JSON/input/text ceilings. Add no budget
  flags in AP56; report effective limits and omissions.
- Return declaration signature/documentation, available implementation, then
  direct nominal types in the signature, receiver and explicit local annotations.
  Use checked metadata on its snapshot-owned AST/retained expression allocations.
  Expand structural wrappers once to
  find nominal arguments; do not recursively traverse members, imports or callees.
- Treat generated editor API declarations as intrinsic declarations. Their
  physical source location may be returned, but stub bodies are unavailable as
  implementation. Source-less builtins also have no physical implementation.
- Label exact excerpts, truncation, omitted sections and unavailable metadata
  separately. Deliberately bounded context does not imply complete transitive
  context or change-impact analysis.

### Q88-Q90: Calls and refactoring

Specify separate direct-call, value/callback and unresolved-site collections,
including argument/parameter mappings and distinct completeness fields. Use
[AP56.5](05-call-relations.md)'s source-form matrix rather than assuming all
checked references are calls.

Specify `fpas refactor` action requests and the versioned plan envelope described
in [AP56.6](06-refactor.md): preview by default, optional UTF-8 `--out`, explicit
`apply --plan`, exact recipe regeneration, full before/after inventory and repeat
outcomes. Compare inventories before resolving an old target on retry;
`result.status: already_applied` is an `ok` result with no writes, while an
eligible empty plan reports `no_change`. A mismatching plan is `stale_plan`.
No-change, unsupported and incomplete are distinct core results. Incomplete
analysis blocks mutation with `incomplete`; unsupported
refactorings and invalid recipes produce structured errors. Preserve Q86's
publication and final-output failure boundaries.

[AP56.7](07-add-parameter.md) fixes supported and blocked declaration/use forms
for append-only insertion with explicit name/type/mode/argument. Parse complete
fragments in their stated scopes. Missing or unsupported call mappings block the plan.
CLI options and emitted existing source forms introduce no FPAS syntax change.

### Q91: Shared core and adapter contracts

Specify typed requests/results and snapshot/policy inputs before their CLI or
LSP projections. Include disk and editor-overlay examples, local editor handles,
portable references, version guards and comparison of equal semantic contexts.
Distinguish scoped interactive analysis from complete inventories required for
portable references and checked mutation.
Specify `editor_catalog` versus `checked_context`, command-backed extract/import
actions and invocation-time revalidation, including a changed unedited dependency.
State the remaining race after a standard edit is delivered to a client.

Specify versioned `fpas/sourceQuery` and `fpas/refactor` extension payloads,
capabilities, standard LSP mappings and edit/version failures. Use the mandatory
[operation mapping](adapter-contract.md#required-operation-mapping); preserve
coverage and ambiguity where standard protocol collections cannot express them.
Worked fixtures must demonstrate both adapter routes, including stale editor
state, Unicode conversion and shared semantic blockers.

## Affected files

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `README.md`, `adapter-contract.md` | Keep Q85-Q91 choices and adapter requirements aligned |
| Modify | `../../agent-source-queries.md` | Align the owning cross-operation contract |
| Modify | `02-foundation.md` through `07-add-parameter.md` | Align contracts and dependencies |

## Acceptance and verification

Q85-Q91 choices agree across the detail plan and work files. Concrete schema and
canonical-encoding examples cover reads, ambiguity, clipping, stale references,
refactoring plans, apply/reapply and publication/recovery errors. Include the
versioned plan recipe, before/after inventory, exact edits and content digest.
Specify field ordering, framing, path identity and unsupported-path outcomes.
Specify shared build inputs and LSP/editor mappings, including typed extension
requests, standard protocol projections and disk-versus-overlay cases. Neither
adapter's payload format may become the core's semantic model.
Validate links, proposed help examples, ownership and `git diff --check`.

The implementer completes remaining technical details autonomously. Keep AP56.1
open until its fixtures are complete, not until a user approves CLI or JSON design.
