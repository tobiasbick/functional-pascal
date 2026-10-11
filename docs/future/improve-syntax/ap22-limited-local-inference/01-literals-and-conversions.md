# AP22.1: Inference from literals and conversions

Package: [AP22: Limited local inference](README.md)

## Scope

Statement-local `const` and `var` bindings can omit a type annotation when the
initializer is a literal or a resolved explicit type conversion.

Literal forms include scalar literals, signed numeric literals, and non-empty
array and dictionary literals with complete, matching types. Parentheses are
transparent. Dictionary keys and values supply separate evidence. Empty
collections, nested empty collections, unresolved generic arguments, and mixed
element types require annotations; inference does not promote numeric types.
Record and enum construction belong to AP22.2.

## Prerequisites

- AP16.3 (binding keywords).
- AP02 (diagnostic codes).
- AP19 (explicit distinct conversions).
- The agreed literal forms in the [package README](README.md).

## Implementation

- Parser: `LocalBinding` owns the optional annotation; declaration-level
  `ConstDef` and `VarDef` retain mandatory annotations.
- Sema: `check/decl/local_inference.rs` validates literal evidence and reuses
  resolved AP19 conversion metadata for `UserId(Value)` and `integer(Id)`.
  Ordinary conversion routines such as `IntToReal(...)` remain ordinary calls.
- Unsupported or ambiguous local initializers report FP3036 with an annotation
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
  shadowed conversion names, fixed mutable types, and rejected initializer forms.
- Compiler/VM and FPAS runner coverage for stored types, static labels, captures,
  and conversion initializer execution order.
- Formatter round trips, language-service hover/completion with shadowing,
  imported nominal type navigation, and unsaved-buffer LSP hover serialization.
- Real CLI JSON diagnostics and imported aliased conversions through fresh,
  reused, and rebuilt compiled units. Private inferred scalar and aggregate
  bodies keep the same explicitly typed exported routine.

## Follow-ups

- AP18 adds subrange conversion inference and its boundary tests when subranges
  are implemented.
