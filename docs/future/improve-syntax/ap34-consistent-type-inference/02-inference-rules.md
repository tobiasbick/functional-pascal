# AP34.2: Consistent inference rules

Package: [AP34](README.md). Status: complete.

## Implemented rules

[AP34.1's selected rules](01-inference-decision.md) infer every fully
determined statement-local initializer, infer generic arguments from supplied
values first, and fill only unresolved parameters from expected types. Reject
conflicts and unknown types; keep signatures and declarations outside statement
lists explicitly typed. Constructors, direct/member/native calls and callable
values use the same evidence rule. Matching collection and decision values supply
joint context. Defaults do not infer arguments; constraints, nominal identity and
evaluation order remain enforced. A receiving task annotation retains AP30's
distinct rule and does not specialize the spawned call.

## Implementation ownership

- `crates/fpas-sema/src/check/decl/local_inference.rs` accepts complete checked
  initializer types. `decl/routine_type_params.rs`, `decl/routine_body.rs` and
  record-method body checking preserve distinct full lexical parameter identities.
- `check/calls/inference.rs`, `result_inference.rs` and `contextual.rs` combine
  value evidence, expected results and nested callable specialization, then check
  supplied arguments against the completed types without implicit conversions.
- `check/expr/construction_inference.rs`, `collections.rs`, `decision.rs` and
  case-expression checking complete constructor, element and branch evidence.
- `check/expr/calls/` and `postfix.rs` pass context to direct, static, instance
  and final postfix calls. Native operations reuse receiver-specialized catalog
  signatures and retain operation-specific checks. Method lookup and checking
  live in separate `calls/methods/lookup.rs` and `calls/methods/mod.rs` modules.
- `check/inference_completion.rs` tracks deferred call, constructor and callable
  sites. Program/unit entry checks reject every unresolved site before lowering,
  even when a condition, closure or outer Boolean hides its incomplete type.
- Existing binding storage, capture lowering, unit interfaces and semantic editor
  symbols consume the checked types. Compiler source fingerprints invalidate
  incompatible derived unit sidecars without changing the bytecode format.

## Dependencies, migration, and documentation

AP34.1 is complete. AP33's explicit numeric conversions remain in force.
Existing valid annotations remain valid. Affected rejection fixtures and the
runnable local-inference example use the current rule; no annotation-removal
sweep is required. Updated documentation includes
[local variables](../../../pascal/language/basics/local-variables.md),
[generics](../../../pascal/language/types/generics.md),
[generic routines](../../../pascal/language/functions/generic-routines.md),
the compact reference, decision expressions, diagnostics and AP22 references.

## Verification

Regression coverage verifies AP34.1's table and covers ordinary calls versus collections,
constructors versus factories, known expression types, empty collections and
payloadless variants, result-only generic factories, direct/callable/member
calls, constructor fields, reordered named arguments, expected types from
returns/assignments/arguments/fields, conflicts, constraints and unit boundaries.
Existing parser coverage retains explicit signature/declaration boundaries.
Source-level negatives preserve known numeric fragments and reject constraint
violations, unused parameters, unspecialized callable payloads and unresolved
inner sites hidden by complete outer expressions. Runtime tests preserve written
argument order, selected-branch execution, captures and fixed storage types.
Editor tests cover inferred hover/completion and imported nominal navigation;
real CLI tests cover diagnostics and fresh/reused/rebuilt compiled units. The
compact reference's complete source examples are checked, run and formatted.

## Open follow-ups

None within this work package.
