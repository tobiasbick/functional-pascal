# AP24.3: Generic enums and recursion

Package: [AP24: Generic data structures](README.md)

Status: complete.

## Scope

Allow user-defined generic enums with payloads, recursive generic types, and
nested matching on them.

## Agreed rules

- Construct variants with the ordinary variant name, such as
  `Lookup.Found('x')` or `Lookup.Missing`. Explicit type arguments belong in
  type declarations and annotations, not variant-constructor expressions.
- Follow the package's [constructor inference rules](README.md#constructor-type-inference):
  all supplied payload fields determine type parameters together; the
  expected type fills missing parameters. Conflicts are errors. Parameters
  left undetermined require an annotation.
- Follow the package's [recursive type-argument rules](README.md#recursive-type-arguments):
  every reference in a recursive generic cycle passes the parameters
  unchanged and in the same positions, including mutual recursion. Apply
  AP11.1 finite-construction checks after enforcing that restriction.

## Prerequisites

- AP24.2 (generic records).
- AP20.2 (nested patterns).
- AP03.2 (closed-enum exhaustiveness rules).
- AP11.1 (finite recursive construction).

## Implementation

- Generic enum headers and applications use `of`. Variant constructors use
  ordinary names; explicit constructor applications receive an annotation hint.
- Records and enum variants share joint constructor inference. Supplied
  payloads and expected applications determine arguments, constraints and
  nominal compatibility are enforced, and nested constructors and generic
  callable payloads receive context after all supplied fields contribute.
- Direct, mutual, and record/enum recursion forwards parameters unchanged and
  in order before AP11.1 checks finite construction. Transformed arguments and
  mandatory stored-value cycles without a terminating path are rejected.
- Nested patterns use instantiated payload types. Ordinary constructor lookup,
  explicit bindings, guards, and AP03 closed-enum exhaustiveness remain in force.
- Named variant construction retains AP09.2 field mapping and written-order
  evaluation. Type inference is independent of named-field order.
- Canonical declarations define erased runtime layouts. Pattern metadata restores
  concrete payload types, including containers and callables. Compiled-unit
  interfaces retain parameters, arguments, aliases, and recursive references.

## Affected areas

- `crates/fpas-parser/src/parser/decl/`: data type parameters and enum headers.
- `crates/fpas-sema/src/types/enums.rs`, `types/substitution.rs`,
  `types/inference.rs`, `check/decl/types/`, and `check/calls/`: nominal
  applications, constraints, recursion, and generic routine evidence.
- `crates/fpas-sema/src/check/expr/construction_inference.rs`,
  `enum_construction.rs`, and `check/stmt/control_flow/if_case/`: constructor
  context, payload types, nested matching, and exhaustiveness.
- `crates/fpas-compiler/src/lowering/case/`, `calls.rs`, and `types/`:
  canonical erased layouts, payload conversion, and typed pattern lowering.
- `crates/fpas-unit/src/interface/` and `crates/fpas-sema/src/interface/`:
  persistent generic enum descriptors and application references.

## Migration

None.

## Documentation

- `docs/pascal/language/types/generics.md`, `enums.md`, `declaration-order.md`,
  and the types overview.
- `docs/pascal/language/pattern-matching/enum-patterns.md`,
  `docs/specs/grammar.ebnf`, and `docs/pascal/tools/fmt-style.md`.

## Verification

- Multiple instantiations, recursion, invalid recursion, nested patterns,
  `Lookup.Missing` without context rejected, imported generic enums.
- Named construction of generic variants, including type-argument inference
  from reordered named fields and imported generic enums.
- Expected types completing partially determined parameters and variants
  without payloads; conflicts between payload fields or with the expected
  type rejected. Explicit constructor type applications rejected.
- Direct and mutual recursion with unchanged parameters accepted when finite
  construction exists; reordered, replaced, or wrapped parameters rejected,
  including cycles with a terminating variant.
- Distinct instantiated payloads, nested missing alternatives, closed-enum
  `else` rejection, callable payloads, equality, and discard safety.
- Fresh and reused sidecars preserve imported enum arguments, recursive
  payloads, source aliases, named evaluation order, and transitive type aliases.
- Parser, semantic, compiler/VM, and real CLI regressions own these contracts;
  the runnable example and `tests/runner/generic_enums_test.fpas` exercise
  the regular runner. Workspace and FPAS suites, source formatting,
  example/app projects, documentation examples and links, and the real
  Extension Host verify the integrated package.
