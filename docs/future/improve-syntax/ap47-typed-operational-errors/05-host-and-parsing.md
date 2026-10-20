# AP47.5: Host, parsing and related Std migration

Status: Complete. Implemented under the user's AP47.5 authorization after AP47.4.
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

## Implementation owners

| Path | Responsibility |
| --- | --- |
| `crates/fpas-sema/src/std_registry/loaded/`, `src/std_registry/api.rs`, `src/interface/install_native_errors.rs` | Native families, signatures and transitive imported error dependencies |
| `crates/fpas-std/src/native_errors.rs`, `src/std_units/`, `src/fs/`, `src/fs.rs`, `src/proc.rs`, `src/parse.rs`, `src/json.rs`, `src/toml/` | Detection, normalization and typed Result payloads; shared native-error catalog and focused domain adapters |
| `crates/fpas-vm/src/vm/hosted/crypto.rs`, `src/vm/hosted/proc.rs`, `src/vm/layouts.rs`, `src/vm/layouts/native_errors.rs` | Host crypto/process outcomes and nominal layout admission once per image |
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


## Delivery evidence

All 32 selected native Result operations and 20 source JSON/TOML field helpers
now expose the selected nominal families. Host failures preserve the supplied
operation/resources and available native domain/code. Parser causes and optional
positions remain structured. Intentional application/SDK string boundaries use
explicit Message() mappings. Task/network interfaces remain scheduled for AP47.6.

The VM validates the required native ABI once per image and caches its layouts;
metadata-only dependencies survive source interface imports. Missing families,
wrong payload types, duplicate owners and invalid Cause metadata fail admission
instead of becoming operational errors. Numeric parsing uses the existing shared
grammar, distinguishing syntax from range only after syntax validation.

A subagent reviewed the implementation and the repairs. Findings concerning
failed filesystem paths, empty process resources, ABI admission, NUL classification
and synthetic host causes were fixed and rechecked. Production-path tests cover
source execution and portable images, parser/field causes, native error details,
file limits, process failures/nonzero exits and secure entropy backend failure.
A controlled denied-access check preserved the native permission code. The same
valid OutOfRange pattern fails on the unchanged control because its payload is
string, and succeeds after migration. Scalar membership uses an exact Integer
comparison; mixed/nested/empty arrays and integer extrema retain their previous
membership semantics.

Validation: 4,887 Rust tests passed with 3 ignored; 534 Pascal tests passed with
1 intentional skip; all 24 affected project/source checks passed. Workspace build,
strict all-target/all-feature Clippy, both formatters, editor grammar and whitespace
checks passed. Regenerating all 20 native API files produces identical output.
AP47.6 and AP47.7 are now complete;
[integrated acceptance](07-acceptance-and-performance.md#integrated-delivery-record)
records the final AP47 closure.
