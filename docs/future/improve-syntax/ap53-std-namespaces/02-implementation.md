# AP53.2: Migrate FPAS and its repository consumers

Package: [AP53](README.md). Status: complete.

## Prerequisites

- [AP53.1](01-decision.md) is complete and Q68-Q71 are recorded.
- The user explicitly authorizes implementation beyond planning/documentation.
- The public-unit and consumer inventory is recorded in the implementation
  owners and migration sections below.

## Contract and scope

The compiler, library and repository consumers follow the
[existing-unit mapping](../../../specs/std-namespaces.md#existing-public-units-and-migration-targets)
together. The bundled library imports the canonical names.

For example, the import `Std.Ai.Acp as Acp` exposes the ACP interface through
`Acp`. The former `Std.Acp` is an unknown standard-unit import.
`Std.Text.Format` owns the four text-formatting functions, while
`Std.Math.IntToReal` preserves numeric conversion. Mixed callers
use both imports.

Keep alias rules, case-insensitive matching, access checks, signatures, runtime
behavior and failure categories. Update nominal public identities and displayed
operation names consistently. Existing private implementation units remain
private. A parent namespace grants neither imports nor exports of its children.

Move current TCP/TLS functions together to `Std.Net.Sockets` and the terminal
UI together to `Std.Ui.Terminal`. This package does not redesign sockets or UI,
split the existing Base64/hex functions, or implement any reserved future unit.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `crates/fpas-builtins/src/std_units/` | Canonical names, lookup, symbol ownership and hints |
| Modify | `crates/fpas-sema/src/std_registry/` | Intrinsic/source registration, public type identities and signatures |
| Audit and modify where affected | `crates/fpas-compiler/`, `crates/fpas-code/`, `crates/fpas-unit/`, `crates/fpas-binary/` | Intrinsic lookup, lowering, interfaces, artifacts and invalidation |
| Modify | `crates/fpas-runtime/` | Public operation labels, dispatch mappings and embedded fixtures |
| Modify | `crates/fpas-cli/`, `crates/fpas-lsp/`, `crates/fpas-fmt/` | Import resolution, docs lookup, editor actions, formatting and fixtures |
| Move/modify | `lib/Std/`, `lib/stdlib.fpasprj` | Source-unit identities, imports, private children and public exports |
| Regenerate | `lib/api/Std/` | Canonical editor declarations; remove stale generated paths |
| Modify | `apps/`, `examples/`, `tests/`, `tools/`, `scripts/` | All affected callers, manifests, benchmark/source generators and expected diagnostics |
| Modify where affected | `editors/`, `.agents/skills/` | Templates, snippets, completion, highlighting and authoring guidance |
| Modify | `docs/pascal/`, comments in `docs/specs/grammar.ebnf` | Delivered imports and examples, with unchanged namespace syntax |
| Reconcile | `docs/future/`, `docs/specs/std-namespaces.md` | Current-versus-target wording and future references after delivery |

Use the [shared implementation touchpoints](../../../pascal/std/README.md#shared-implementation-touchpoints)
and search all tracked sources for old qualified paths. The table names owners,
not permission to modify unrelated behavior or mechanically rename all internal
Rust symbols. Confirm exact files and file-size boundaries before implementation.

## Migration and artifacts

The refreshed inventory includes these concrete owners in addition to the table:
`fpas-builtins/src/{aggregates.rs,native_records.rs,native_errors.rs,std_units/}`,
`fpas-sema/src/{std_registry/,check/,interface/,tests/}` and
`fpas-sema/examples/export_intrinsic_std_api/`; compiler intrinsic lookup and
runtime labels; `crates/fpas-build/build/compiler_identity.rs`; CLI import, docs,
visibility and artifact tests; `fpas-language-service`, `fpas-project`, and the
VS Code README. The generated editor API is rebuilt from the migrated registry.

`crates/fpas-compiler/src/lowering/closures/conversion_values.rs` supplies
callable wrappers for formatting and numeric conversion routines, alongside
the existing parsing wrappers. Imported routine values retain their owning
unit, signature and argument order.

`crates/fpas-code/src/lib.rs` advances the intrinsic ABI to bytecode version 29
because native nominal type identities changed. The existing program decoder
rejects earlier images before execution, and sidecar identity checks rebuild
old artifacts. Program/unit envelope layouts and object schemas are unchanged.
`crates/fpas-program/tests/format.rs` covers rejection of the preceding ABI;
its golden image and `crates/fpas-bundle/tests/format.rs` reflect the changed
embedded program identity. The compiled-program handbook describes the rebuild
contract.

Source families move from `lib/Std/Calendar` to `lib/Std/Time/Calendar`,
`Http` to `Net/Http`, `Tui` to `Ui/Terminal`, `Net/Utf8.fpas` to
`Text/Utf8.fpas`, and `Json/Fields.fpas` and `Toml/Fields.fpas` beneath
`Encoding/`. This includes each family's `.fpas` entry point and private source
children. Ignored old sidecars are not moved into the new source tree.
The text conversion handbook becomes `docs/pascal/std/text/format.md`;
numeric conversion belongs in the Math handbook. All affected Markdown and
grammar examples, top-level README and distribution paths are reconciled.

- Migrate code and public signatures together, including aliases used for
  types, enum variants, routine values and calls inside embedded fixtures.
- Update imports semantically where one old unit splits into two. Ordinary
  string payloads containing names are changed only when they represent an API
  identity, diagnostic expectation, command or documentation example.
- Rebuild source-adjacent interfaces and executable artifacts with the current
  compiler. Verify invalidation of artifacts referring to removed unit names.
  Do not add a compatibility reader solely to preserve the old namespace map.
- Preserve the export manifest's private/public distinction. Directory layout
  and generated filenames must agree with declarations and tooling lookup.
- Update handbook pages, Rust handbook links, `fpas doc`, generated declarations,
  the compact agent reference and distribution staging together.

## Acceptance and verification

Positive cases cover each relocated family, mixed formatting/math imports,
case-insensitive imports, nominal types in public signatures and private source
dependencies. Negative cases reject old names, unknown children and private
units, including through parents or aliases. Preserve duplicate-import and
alias-collision diagnostics.

Exercise CLI checks/runs/docs, LSP completion/auto-import/navigation, formatter
round trips, fresh/reused sidecars and standard-library distribution staging.
Reuse existing import, visibility, artifact, numeric-text and standard-library
tests before adding coverage for a distinct missing regression.

Existing coverage owners include `crates/fpas-cli/src/main_tests/visibility/`,
`crates/fpas-cli/src/main_tests/diagnostics/imports.rs`,
`crates/fpas-cli/src/main_tests/fmt/handbook_imports.rs`,
`crates/fpas-sema/src/tests/integration/std_units/`,
`crates/fpas-sema/src/interface/tests/import_aliases.rs`,
`tests/stdlib/imports/aliases_test.fpas` and
`tests/stdlib/conv/numeric_text_contracts_test.fpas`.

Use the repository's
[required checks](../development-process.md#required-checks): Rust formatting,
development build with `--profile dev-fast`, workspace tests, FPAS formatting,
the FPAS suite and affected application/example checks. Use Fat-LTO for any
release, benchmark or dist build. Run editor checks when those files change.
Document-only validation in AP53.1 is not evidence that these checks passed.

The inventory has no unhandled old API uses. Old spellings remain only in
intentional negative fixtures or explicitly identified migration descriptions.
Applicable checks and the Spec/Standards reviews are complete; commands,
results and coverage limits are in the
[verification report](../../../reports/ap53-namespace-migration.md).
The delivered compiler/library pair is the dependency of
[AP53.3](03-dungeon-migration.md).
