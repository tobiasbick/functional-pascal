# AP53.2: Migrate FPAS and its repository consumers

Package: [AP53](README.md). Status: not started; implementation not authorized.

## Prerequisites

- [AP53.1](01-decision.md) is complete and Q68-Q71 are recorded.
- The user explicitly authorizes implementation beyond planning/documentation.
- Refresh the public-unit and consumer inventory against the implementation
  checkout. Record any additional affected paths before editing them.

## Contract and scope

Implement the [existing-unit mapping](../../../specs/std-namespaces.md#existing-public-units-and-migration-targets)
as one consistent compiler/library and in-repository consumer migration.
Do not ship an intermediate compiler whose bundled library imports old names.

For example, the target import `Std.Ai.Acp as Acp` exposes the current ACP
interface through `Acp`. `Std.Acp` becomes an unknown standard-unit import.
`Std.Text.Format` owns the four current text-formatting functions, while
`Std.Math.IntToReal` preserves the current numeric conversion. Mixed callers
must use both imports. These are target examples until this work is delivered.

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

When implementation is authorized, use the repository's
[required checks](../development-process.md#required-checks): Rust formatting,
development build with `--profile dev-fast`, workspace tests, FPAS formatting,
the FPAS suite and affected application/example checks. Use Fat-LTO for any
release, benchmark or dist build. Run editor checks when those files change.
Document-only validation in AP53.1 is not evidence that these checks passed.

Acceptance requires an inventory with no unhandled old API uses, passing
applicable checks and a Spec/Standards review. Allow old spellings only in
intentional negative fixtures or explicitly identified migration descriptions.
Then mark AP53.2 complete in the package README and hand the delivered compiler
and library to [AP53.3](03-dungeon-migration.md).
