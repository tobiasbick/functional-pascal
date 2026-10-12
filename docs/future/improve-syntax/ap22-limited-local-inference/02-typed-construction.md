# AP22.2: Inference from typed construction

AP34 supersedes the restriction that ordinary factory calls require annotations.
See [AP34.2](../ap34-consistent-type-inference/02-inference-rules.md) for current rules.

Package: [AP22: Limited local inference](README.md)

Status: complete.

## Scope

Statement-local bindings can omit a type annotation when the initializer is a
resolved record or enum construction, for example
`const P := Point(X := 1, Y := 2);` or `const S := Shape.Circle(2.0);`.
AP34 applies the same eligibility rule to their factories and member access.

## Prerequisites

- AP24 (generic record and enum constructor inference).
- AP22.1 (local inference rules).
- AP10.1 (typed construction).
- The agreed inclusion of enum variant constructors in the [package README](README.md).

## Implementation

- Sema reuses resolved record-construction metadata and records enum call
  constructor identity. Payloadless enum members use resolved designator
  metadata; shadowed type or variant names do not become constructors.
- Record fields and enum payloads determine generic applications under AP24's
  rules. Defaults do not infer arguments. A fixed type alias can supply the
  arguments; otherwise missing arguments require an annotation.
- Non-generic payloadless enum variants infer their nominal type. `Some(Value)`
  infers a complete Option payload; lone `None`, `Ok(Value)`, and `Error(Value)`
  require context. AP34 joins matching sibling and expected-type evidence.
- Imported types and exported enum aliases retain qualified constructor lookup
  without replacing the original generic type's variant symbols.
- Checked initializer types feed the existing compiler storage/capture path and
  semantic binding metadata used by editor hover, completion, and navigation.

## Affected areas

- `crates/fpas-sema/src/check/decl/local_inference.rs`, `check/context.rs`, and
  `check/expr/calls/mod.rs`.
- `crates/fpas-sema/src/interface/install.rs` for exported enum alias variants.
- Sema, compiler/VM, language-service, CLI, and FPAS runner regression coverage.

## Migration

None; explicit annotations remain valid.

## Documentation

- `docs/pascal/language/basics/local-variables.md`, `constants.md`.
- `docs/pascal/language/types/records.md`, `enums.md`.
- `docs/pascal/tools/diagnostics.md` and the runnable inference example.

## Verification

- Sema covers concrete and generic records/enums, fixed aliases, nested
  constructors, payloadless variants, Option payloads, ordinary factories and
  postfix access, shadowing, fixed nominal types, original
  argument diagnostics, underconstrained constructors, and generic routine
  payloads that require a concrete callable signature.
- Inferred aggregates preserve task-handle and callable discard restrictions.
- Compiler/VM tests cover record capture cells, copied enum captures, nested
  layouts, written argument order, omitted defaults, and initializer execution
  at the reached statement.
- Language-service tests cover concrete imported hover/member completion and
  nominal type navigation through an import alias.
- Real CLI builds exercise imported constructors and fixed aliases through
  fresh, reused, and rebuilt units with explicitly typed public signatures.
- The FPAS runner and runnable example exercise unannotated constructors.
