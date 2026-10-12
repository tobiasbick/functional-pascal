# AP31.1: Persist evaluated constant defaults

Package: [AP31](README.md). Status: complete.

## Result

Exported record defaults use values retained by the existing compile-time
constant classification and evaluator at the declaration. The interface
serializer consumes these checked values. Imported defaults preserve the
same values through aliases and generic instantiation, independent of caller
names and fresh or reused sidecars.

Unsupported computed or aggregate defaults, and constant expressions without
a valid value, produce FP3014 at the original field expression before interface
extraction. Transparent aliases share retained expression identities and do
not duplicate that diagnostic. Local and non-exported defaults keep their
existing declaration-scope behavior.

## Implementation ownership

- `crates/fpas-sema/src/check/decl/types/records/defaults.rs` and
  `check/decl/consts/` own declaration-scope checking and evaluated constant values.
- `crates/fpas-sema/src/interface/export/record_defaults.rs` owns source diagnostics
  and persistence of those values; `interface/analysis.rs` runs validation before
  interface extraction.
- `crates/fpas-sema/src/interface/export.rs` qualifies unit-owned enum defaults.
- `crates/fpas-sema/src/interface/install.rs` retains imported values and stable
  expression allocations, including optional `None` and canonical enum identity.
- `crates/fpas-unit/src/interface/types.rs` supplies the existing scalar and
  optional-`None` field-default representation.

## Migration and documentation

No source-syntax migration. The
[record default reference](../../../pascal/language/types/records.md#default-field-values)
describes declaration-evaluated values and their export boundary.
[Diagnostic guidance](../../../pascal/tools/diagnostics.md) includes exported
defaults under FP3014.

## Regression coverage

- `crates/fpas-build/tests/record_construction/mod.rs` exercises literal and
  expression equivalence, preceding and imported constant references, parentheses,
  unary minus, real/string/Boolean values, signed integer boundaries, short-circuit
  evaluation, enum values, and optional callable `None`. It checks local defaults,
  explicit overrides, transparent aliases, generic records, caller shadowing,
  and identical execution with fresh and reused sidecars.
- `crates/fpas-build/tests/diagnostics/record_defaults.rs` exercises unsupported
  routine calls, mutable and computed references, aggregate defaults, and failing
  integer/real divisions through the public build and check APIs. It checks the
  diagnostic code, declaring file, source ID, field line, useful hint, and absence
  of a published invalid sidecar.

## Open follow-ups

None within this work package's scope.
