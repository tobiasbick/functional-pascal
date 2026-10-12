# AP34.2: Consistent inference rules

Package: [AP34](README.md). Status: blocked by AP34.1.

## Scope and affected areas

Implement only the recorded decision. Inspect
`crates/fpas-sema/src/check/decl/local_inference.rs`,
`check/calls/inference.rs`, `check/expr/construction_inference.rs`, callable
instantiation, diagnostics, unit interfaces, and language-service type display.
Preserve constraint checking and reject unresolved types before lowering.

## Dependencies, migration, and documentation

AP34.1 is required; AP33 is not a prerequisite unless the chosen algorithm
explicitly relies on changed numeric compatibility. Add/remove annotations
only as required by the chosen rule across all consumers. Update
[local variables](../../../pascal/language/basics/local-variables.md),
[generics](../../../pascal/language/types/generics.md),
[generic routines](../../../pascal/language/functions/generic-routines.md),
the compact reference, diagnostics, and affected AP22 decision references.

## Verification

Test known and ambiguous scalar/collection initializers, generic result-only
factories, direct/callable/member calls, constructor fields, reordered named
arguments, return/assignment contexts, conflicts, and unit boundaries. Ensure
public signatures stay explicit and adding an annotation cannot silently
select another value conversion. Run required implementation checks.
