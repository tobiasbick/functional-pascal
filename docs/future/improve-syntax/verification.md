# Verification and migration

This document supports the [six stages](README.md). It is not another backlog or
completion index. Tests check language behavior and tooling; no LLM benchmarks,
model performance thresholds, or model-based acceptance criteria are required.

## Per-slice delivery

1. Refresh source, tests, branch state, and the stage's current evidence. Identify
   what already works before replacing it. Inspect module size and directory shape.
2. Classify the change and list concrete paths to create/change/move/remove.
   Split only the oversized concerns affected by this work; avoid generic helpers.
3. Select one coherent slice, including dependent migration. Record expected
   behavior and negative cases before implementation.
4. Implement through the real parser, sema, compiler, bytecode verifier, VM, and
   CLI paths where applicable. Do not test only a helper that bypasses integration.
5. Update current docs, grammar, Rust doc links, generated declarations, formatter,
   editor signatures/snippets, and source consumers for implemented behavior.
6. Run appropriate checks, record actual results/limits, and update the owning
   stage's status, evidence, blockers, and next step. Check the central stage box
   only when every required slice and its migration are complete.

An approved direction is not a passed test. A parsed example is not evidence of
correct execution, and one passing fixture is not a complete source migration.

## Migration boundaries

Inventory `.fpas` consumers under `lib/`, `apps/`, `examples/`, and `tests/`, plus
Rust-embedded sources, golden files, CLI/project templates, generated intrinsic
declarations, editor integrations, repository skills, and handbook examples.
Project manifests/exports may need updates when methods move to unit routines.
Do not assume syntax migration is limited to files with a `.fpas` extension.

Use temporary syntax- and symbol-aware conversion tooling when mechanical edits
cannot preserve meaning. In particular:

- Method and receiver calls require their resolved target, receiver evaluation
  order, and true caller-mutation behavior.
- Record literals require their contextual type and original field values.
- Binding migration must preserve local reassignment versus caller mutation,
  captured cells, resource identity, and constant-only contexts.
- Changed boolean precedence needs explicit grouping of the original intended
  expression, with evaluation count and short-circuit behavior verified.
- Imports need resolved unit ownership and collision checks, including variants
  and type names, rather than textual prefix insertion.
- Task migration must give long-lived work an explicit owner, account for every
  function result, and remove alternate spawning/group lifetime escape hatches.

Keep intended application results and protocols stable where the new model can
express them. Record unavoidable behavior changes from approved semantics rather
than masking them with a compatibility layer or weakened assertion.

The delivered compiler accepts one language. Remove temporary conversion tools
after migration, obsolete AST/bytecode/runtime branches, dead imports/modules,
stale docs, and superseded tests. Preserve negative tests for the old spelling as
diagnostic cases. Never hand-edit or commit derived `.fpascu` artifacts.

## Interaction coverage

| Area | Required positive, negative, and boundary evidence |
|------|------------------------------------------------------|
| Grammar | Every block kind, nested closers, final terminators, empty bodies, comments, expressions inside arguments, formatter round trips and idempotence |
| Names | Case-insensitive collisions, alias-only imports, qualified variants, non-public fields, forward types, declaration-order initializers |
| Calls | Returned/indexed/selected/captured callables, callable fields, no implicit receiver, positional-only arguments, defaults only on records |
| Inference | Ordinary calls and expressions, explicit expected types, empty collections, underdetermined constructors, no inference from later assignments |
| Data | Nested copy isolation, record update order, constructor defaults, resource identity inside values, structural array/dictionary equality, unsupported nested equality |
| Generics | Multiple/nested instantiations, finite recursive data, rejected alias/layout cycles, constraints, generic routine values with expected signatures |
| Patterns | Explicit bindings versus constants, nested coverage, guard-only gaps, grouped-label bindings, enum extension, forbidden enum catch-all, open-domain expression fallback |
| Mutation | Direct/indirect var calls, evaluated-once indices, duplicate/forwarded roots, callback aliases, invalid roots, snapshot arguments, partial changes on failure |
| Purity | Pure callback composition, capability conversion direction, captures, local mutation, resource-containing types, generic instantiation, std metadata |
| Evaluation | Left-to-right effects, short-circuiting, mixed logical operators, integer limits/overflow, zero division, shift counts, real NaN/infinity behavior |
| Results | Unused Result/Option, explicit discard, procedure misuse, nested task-handle discard rejection, try forwarding, no panic-to-Result conversion |
| Tasks | Lexical owners, both go forms, every exit path, nested cancellation, no survivors, blocked children, helper/container escape, result observation, repeated waits, child panic arbitration |
| Domain rules | Distinct identities, subrange endpoints/overflow/conversions, unchanged resource restrictions under wrappers |
| Contracts | Clause scopes, pure calls, every normal return including Result errors, no postconditions on panic, cleanup order, mandatory release checks |
| Tooling | Shared diagnostic codes, Unicode ranges, missing source locations, JSON streams, four CLI commands, actual runner errors, editor/CLI parity |

Prefer existing coverage and add tests for uncovered contracts. Do not write tests
that merely reproduce the implementation or remove unrelated tests to hide errors.
Malformed FPAS fixtures should be Rust-embedded or isolated from the valid-source
formatter corpus rather than silently breaking that corpus.

## Required checks for implementation

Run from the repository root with the appropriate local toolchain:

```text
cargo fmt
cargo build
cargo test --workspace
fpas fmt --check lib/ apps/ examples/ tests/
fpas test tests/suite.fpasprj
git diff --check
```

Use the repository's formatter script where appropriate. Run targeted compiler,
runner, and release-mode cases in addition when the changed contract needs them.
Test source-mutating programs under `.temp-data/`; do not write scratch fixtures
into crates or bare working-directory filenames. Do not add CI workflows.

For a plan-only edit, validate relative links/anchors, examples against the target
contract, document consistency, and the diff. Builds and runtime tests do not
verify unimplemented target syntax and are unnecessary for that edit.

## Completion report

Report implemented scope, current docs touched, tests added/run, migration
coverage, actual verification results, and remaining limitations. Distinguish
design completion, implementation completion, and user acceptance. Keep all
planning documents in English and omit personal/machine metadata and dated logs.
