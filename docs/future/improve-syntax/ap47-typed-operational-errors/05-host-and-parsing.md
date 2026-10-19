# AP47.5: Host, parsing and related Std migration

Status: Planned; implementation not authorized. Depends on AP47.4.
Selected interfaces: [host/parsing catalog](standard-errors.md#host-and-parsing-cohort).
Completion: [AP47](README.md#dependencies-and-work-packages).

## Deliverable

Switch the current Result-returning Fs, Proc, Parse, Json, Toml and Crypto
interfaces and Json/Toml field helpers to the selected native error families.
Add the shared HostError leaf and error-value factories. Replace each signature in
place together with every affected call site, generated declaration and owning
handbook entry. There is one public function per operation, not ReadTextTyped
beside ReadText or a legacy string-returning switch.

## Implementation plan

1. Register Std.Errors.HostError and each native cohort family once in the existing
   intrinsic unit/type catalogs. Export their nominal family kind and static
   messages through ordinary interface metadata. Source field-helper families
   are declared in their own FPAS modules.
2. Add one shared native-error construction module that maps structured backend
   outcomes to admitted shared enum layouts. Keep classification before error
   stringification: the current fs result_string/result_bool helpers and parse
   err(String) paths must no longer erase classification and native codes.
3. Change native signatures and error construction as one coherent slice per
   domain, then migrate all transitive Rust/FPAS callers before its delivery.
   Regenerate lib/api/Std. Preserve success types, limits and runtime diagnostics.
4. Source field helpers wrap parser causes with MapError and construct their own
   structural variants at the existing validation site. Preserve strict unknown
   fields, integer/real validation and helper return shapes.
   Primitive numeric text parsers currently erase both malformed syntax and
   range failure to Option. Refactor their single shared implementation to return
   a structured syntax/range reason, update every Rust caller and preserve the
   existing lexer/compiler literal diagnostics. Finish syntax validation before
   classifying range, so an overflowing prefix with invalid trailing text is
   InvalidSyntax. Do not add another numeric parser in Std.Parse.
5. Update apps, examples, tests, benchmark generators, CLI templates and embedded
   source fixtures. A function intentionally returning an unselected application
   string error uses explicit mapping to Message(); do not insert a global
   compatibility coercion. An affected selected-domain helper adopts its domain
   family instead of flattening causes to text.

Each cohort may leave other domains on their current interfaces until AP47.6;
it may not leave uncompilable callers or introduce temporary exported aliases.
Rebuild precompiled libraries when public type metadata changes. Do not alter
Rust compiler-internal error handling just because the same spelling Result is
used there.

## Actual owners and planned edits

| Path | Responsibility |
| --- | --- |
| `crates/fpas-sema/src/std_registry/loaded/`, `src/std_registry/api.rs` | Native families and signatures |
| `crates/fpas-std/src/std_units/`, `src/fs/`, `src/fs.rs`, `src/proc.rs`, `src/parse.rs`, `src/json.rs`, `src/toml/` | Detection, normalization and typed Result payloads; create focused native-error helpers beside existing owners |
| `crates/fpas-vm/src/vm/hosted/crypto.rs`, `src/vm/hosted/proc.rs`, `src/vm/layouts.rs` | Host crypto/process outcomes and nominal layout admission once per image |
| `crates/fpas-numeric/src/text.rs`, `src/error.rs`, numeric consumers in lexer/compiler | Structured text failure reasons from the same grammar, preserving existing literal diagnostics and arithmetic error behavior |
| `lib/api/Std/` | Regenerated native declarations, including new Std.Errors |
| `lib/Std/Json/Fields.fpas`, `lib/Std/Toml/Fields.fpas` | Source-defined field errors and parser wrapping |
| `lib/Std/`, `apps/`, `examples/`, `tests/`, embedded Rust fixtures | All transitive callers of migrated interfaces |
| `docs/pascal/std/host/`, `std/text/`, `std/cryptography/`, compact reference | Publish the delivered contracts and migration examples |

## Acceptance and verification

Use the production APIs for missing files, denied access where controllable,
invalid patterns/encoding, file limits, temp/metadata operations, process launch
failure and successful nonzero exits. Assert typed variants and preserved native
leaf data; only the output owner asserts complete formatted text.

Parser boundary tables distinguish malformed syntax from numeric range failure
without changing accepted numeric/boolean grammar. JSON/TOML fixtures prove
optional positions, original parser causes, missing/wrong/unknown fields and
nonfinite/invalid values. Crypto preserves secure-source behavior and validation;
backend-failure proof uses its real adapter seam without adding a test-only API.

Known public conditions must fail pre-migration proof for the intended missing
typed contract and pass after delivery. Reuse existing integration and snapshot
owners rather than copy tests into every compiler layer. Exact diagnostics
coverage remains separate from typed decision coverage.

All host/parsing signatures and nested function types match the catalog.
Regeneration is reproducible; all affected projects check and the repository
language/Std suite passes. Run required Rust/workspace checks and benchmark
comparisons; package-specific measurements are governed by AP47.7. Native
filesystem/process checks run with the permissions required for their actual
condition; sandbox-denied host access is not proof of an implementation failure.
