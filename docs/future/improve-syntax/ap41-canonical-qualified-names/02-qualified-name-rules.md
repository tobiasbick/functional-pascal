# AP41.2: Qualified-name rules

Package: [AP41](README.md). Status: complete; implemented and independently reviewed.
Prerequisite: [AP41.1's recorded contract](01-qualification-decision.md).

## Scope and affected areas

Implement mandatory alias imports and user-enum type qualification together
according to AP41.1. This is compiler enforcement rather than an optional lint.

- Require an explicit alias on every direct `uses` entry. Remove plain-import
  installation and short/full-original-path source exposure; preserve explicit
  imports, public visibility, canonical dependency identity, and contextual `as`.
- Retain alias uniqueness, namespace reservations, non-shadowing, and one direct
  import per canonical unit. Keep aliases source-local and non-reexported.
- Register/resolve user variants through their visible type qualifier only.
  Require `Type.Variant` locally and `Alias.Type.Variant` for imported types.
  Remove short variant registration and expected-type shortcuts in expressions,
  comparisons, constructors, case labels, nested patterns, and `is` tests.
- Keep Some/None/Ok/Error as the existing reserved language forms. Retain local
  names, explicit type aliases, generic inference, field-list styles, native/record
  operations, and AP39 coverage; qualification must not change resolved semantics.
- Give concrete alias/qualification repairs for removed forms and collisions.
  Do not add compatibility paths, implicit imports, or receiver-call alternatives.
- Align completion, auto-imports, rename, references, hover, signatures, semantic
  tokens, grammar highlighting, templates, and snippets. Reuse existing aliases;
  new auto-imports default to the final unit segment and select a readable valid
  alternative on conflict without changing existing aliases.

Inspect parser/import AST and grammar consumers;
`crates/fpas-sema/src/check/name_resolution/`, import and enum registration,
compiled-unit export/import installation and type identity, compiler lowering,
formatter handling, and language-service import/name resolution. Remove obsolete
plain-import/short-variant branches and bookkeeping rather than preserving a
parallel legal form. Check artifact compatibility so reused interfaces cannot
restore the removed source spellings; keep source and reused-unit behavior equal.

## Migration and documentation

Migrate lib/apps/examples/tests, Rust and documentation fixtures, benchmarks,
generated declarations/templates, skills, and editor snippets by resolved targets.
Supply an alias per direct import, qualify imported types/routines/constants/
variables, and give every user enum variant its type qualifier. Preserve builtin
forms, local names, explicit type aliases, case-insensitive identity, and ordinary
local declaration shadowing without weakening alias reservations.

Update
[units](../../../pascal/program-structure/units.md),
[enums](../../../pascal/language/types/enums.md), grammar when affected,
compact reference, diagnostics, editor guidance, templates, snippets, and authoring
skills in the same delivery. Mark the affected AP05 decision as replaced while
retaining its completed status and preserved alias/editor behavior. Keep AP20,
AP24/AP34, AP36, and AP39 contracts intact.

## Verification

Turn AP41.1's accepted/rejected matrix into boundary regressions. Reject plain
imports, imported short/original-path names, and local/imported short user variants
with concrete repairs; accept the single required form in values, public types,
routine values, tasks, constructors, comparisons, nested patterns, and `is` tests.

Cover later colliding enum declarations, type/variant collisions, colliding units,
all alias reservations and case variations, generic variants and explicit type
aliases, local declarations with an imported spelling, and retained built-in/native
operations. Exercise source units, serialized/reused interfaces, and incremental
builds; alias rename must preserve canonical type/dependency identity.

Verify editor completion and auto-import generate only legal qualified forms,
reuse existing imports with comments/case variations, choose valid aliases for
colliding or single-segment unit names, and preserve symbol identity during rename.
Check generic and named patterns without changing AP36/AP39 semantics.

Run required Rust/FPAS suites, formatting, example/app checks, editor/grammar
checks, documentation links/examples, and shared syntax regressions. Mark complete
only after compiler checks, migrated consumers, documentation, and tooling agree.

## Delivered implementation

- Parser FP2022 requires an explicit alias. Sema FP3043 identifies removed short
  imported names and user variants, with repairs using the visible alias/type.
  Original imported paths remain hidden; ordinary local declarations and the
  reserved Some/None/Ok/Error forms retain their behavior.
- `scope/imports.rs` separates exact source alias bindings from canonical type
  metadata. Direct interfaces expose their own public exports; supporting
  interfaces provide metadata without opening source names. Native registration
  follows the same separation. Compiled-unit format 18 invalidates old sidecars;
  canonical linked identities and alias-rename reuse remain stable.
- Enum registration and expression/pattern resolution require the visible type
  qualifier. Generic and explicit type aliases, named/positional fields, nested
  patterns, scalar comparisons, and coverage retain their existing semantics.
- Completion, auto-imports, references, rename, signatures, semantic tokens,
  templates, snippets, and generated native editor declarations use the required
  source forms. New imports cannot invalidate a written existing alias through
  a new unit root. Program headers do not compete with imported namespaces.
- Std sources, apps, examples, tests, Rust/editor fixtures, documentation, and
  authoring guidance are migrated. AP05's replaced plain-import decision is
  recorded while its completed alias and editor contracts remain intact.

## Review and validation

Independent review found and verified fixes for new-import root collisions,
program-header/alias editor identity, incorrect or incomplete handbook migration,
and stale diagnostic repair forms. Boundary regressions cover those failures.
Debugger protocol tests also retain immediate launch events, keeping their
existing event-order and atomic-replacement assertions reliable.

Validation on 2026-10-14:

- Rust formatting and workspace build pass. The full workspace test run includes
  765 semantic and 298 compiler unit tests, source/cache integration tests, and
  editor/LSP protocol suites. The previously recorded VM socket-timeout rounding
  failure remains; the confirming run excludes only the two known baseline TCP
  timing/cancellation cases.
- The FPAS suite passes 511 tests with one intentional skip. All 120 app/example
  project and standalone-program checks pass; FPAS source formatting is clean.
- Release binaries and all 111 bundled standard-library units build. Generated
  native declarations parse and format with local or explicitly aliased types.
- Editor TypeScript compilation, grammar checks, and contract checks pass.
  Document links resolve (1,971 relative links in 317 documents), and the reviewed
  enum/type-alias/exhaustiveness examples compile with their intended outcomes.

The baseline exceptions are
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` and
`tcp_backpressure_can_time_out_and_cancel_without_closing`, already reproduced
before AP41. Their implementations and tests are unchanged by this package.
