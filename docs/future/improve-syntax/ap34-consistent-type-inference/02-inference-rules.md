# AP34.2: Consistent inference rules

Package: [AP34](README.md). Status: decision recorded; implementation not started.

## Scope and affected areas

Implement [AP34.1's selected rules](01-inference-decision.md): infer every fully
determined statement-local initializer, infer generic arguments from supplied
values first, and fill only unresolved parameters from expected types. Reject
conflicts and unknown types; keep signatures and declarations outside statement
lists explicitly typed. Inspect
`crates/fpas-sema/src/check/decl/local_inference.rs`,
`check/calls/inference.rs`, `check/expr/construction_inference.rs`, callable
instantiation, diagnostics, unit interfaces, and language-service type display.
Preserve constraint checking, nominal identity and evaluation order, and reject
unresolved types before lowering. Apply the same inference rule to constructors,
direct/member calls and callable values; defaults do not supply type evidence.

## Dependencies, migration, and documentation

AP34.1 is complete. Preserve AP33's explicit numeric conversions without
introducing numeric promotion. Existing valid annotations remain valid; migrate
only affected consumers and rejection fixtures, without a general removal sweep.
Update
[local variables](../../../pascal/language/basics/local-variables.md),
[generics](../../../pascal/language/types/generics.md),
[generic routines](../../../pascal/language/functions/generic-routines.md),
the compact reference, diagnostics, and affected AP22 decision references.

## Verification

Verify AP34.1's prediction table. Cover ordinary calls versus collections,
constructors versus factories, known expression types, empty collections and
payloadless variants, result-only generic factories, direct/callable/member
calls, constructor fields, reordered named arguments, expected types from
returns/assignments/arguments/fields, conflicts, constraints and unit boundaries.
Ensure signatures and declarations outside statement lists stay explicit, and
adding an annotation cannot silently select another value conversion. Run
required implementation checks and validate language-service inferred type display.
