# AP22.1: Inference from literals and conversions

AP34 supersedes this package's initializer-form restriction. Current behavior is
defined by [AP34.2](../ap34-consistent-type-inference/02-inference-rules.md).

Package: [AP22: Limited local inference](README.md)

Status: complete.

## Scope

Statement-local `const` and `var` bindings can omit a type annotation when the
initializer is a literal or a resolved explicit type conversion.

Literal forms include scalar literals, signed numeric literals, and non-empty
array and dictionary literals with complete, matching types. Parentheses are
transparent. Dictionary keys and values supply separate evidence. Empty
collections and unresolved generic arguments require context. AP34 allows
matching sibling elements to complete nested empty collections. Mixed numeric
elements are rejected even with annotations; inference does not promote types.
Record and enum construction belong to AP22.2.

## Prerequisites

- AP16.3 (binding keywords).
- AP02 (diagnostic codes).
- AP19 (explicit distinct conversions).
- The agreed literal forms in the [package README](README.md).

## Implementation

- Parser: `LocalBinding` owns the optional annotation; declaration-level
  `ConstDef` and `VarDef` retain mandatory annotations.
- Sema: `check/decl/local_inference.rs` validates complete checked types and reuses
  resolved AP19 conversion metadata for `UserId(Value)` and `integer(Id)`.
  `IntToReal(...)` remains an ordinary call whose known result is now eligible.
- Underdetermined local initializers report FP3036 with an annotation
  hint. Invalid expressions retain their original diagnostics.
- Compiler: unannotated locals use the checked initializer type for storage,
  including mutable capture cells and nested routine captures.
- Program and unit declarations, declarations before a routine's `begin`,
  parameters, results, and fields stay annotated.
- Language service: semantic binding types supply declaration and use hover
  and completion details. The formatter preserves omitted annotations.

## Affected areas

- `crates/fpas-parser/src/ast/bindings.rs` and local statement parsing/recovery.
- `crates/fpas-sema/src/check/decl/`, shared binding checks and capture preparation.
- Compiler statement lowering and project source-map traversal.
- Formatter statement emission, language-service semantic symbols, diagnostics.

## Migration

None; annotations stay valid.

## Documentation

- `docs/pascal/language/basics/local-variables.md`, `constants.md`,
  `docs/pascal/tools/diagnostics.md`, `docs/specs/grammar.ebnf`.

## Verification

- Parser coverage for optional local annotations, mandatory declaration-level
  annotations, and recovery after a missing initializer.
- Sema coverage for each literal kind, nested collection evidence, conversions,
  shadowed conversion names, fixed mutable types, and unresolved initializers.
- Compiler/VM and FPAS runner coverage for stored types, static labels, captures,
  and conversion initializer execution order.
- Formatter round trips, language-service hover/completion with shadowing,
  imported nominal type navigation, and unsaved-buffer LSP hover serialization.
- Real CLI JSON diagnostics and imported aliased conversions through fresh,
  reused, and rebuilt compiled units. Private inferred scalar and aggregate
  bodies keep the same explicitly typed exported routine.

Subrange conversion inference and its boundary tests are delivered in
[AP18.1](../ap18-subrange-types/01-subrange-declarations.md), including imported
types and inferred hover/type navigation.
