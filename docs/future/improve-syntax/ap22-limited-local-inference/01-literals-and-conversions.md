# AP22.1: Inference from literals and conversions

Package: [AP22: Limited local inference](README.md)

## Scope

Allow local `const` and `var` bindings without a type annotation when the
initializer is a literal or an explicit conversion.

## Prerequisites

- AP16.3 (binding keywords).
- AP02 (diagnostic codes).
- The open decision on literal forms in the [package README](README.md).

## Implementation

- Parser: optional type annotation on local bindings only.
- Sema: infer from the agreed literal forms and from explicit conversions,
  including distinct conversions such as `UserId(42)` (AP19); subrange
  conversions join when AP18 lands (its work packages add the inference test
  cases).
- Reject other initializers, empty collections, and ambiguous literals with a
  diagnostic asking for an annotation and showing the inferred candidates
  where useful.
- Unit-level declarations, parameters, results, and fields stay annotated.
- Language service: inferred types in hover.

## Affected areas

- Parser local declarations, `crates/fpas-sema/src/check/decl/vars.rs`,
  `crates/fpas-sema/src/check/decl/consts/`, `fpas-language-service` hover.

## Migration

None; annotations stay valid.

## Documentation

- `docs/pascal/language/basics/local-variables.md`, `constants.md`,
  `docs/specs/grammar.ebnf`.

## Verification

- Each literal kind and conversion; rejected ordinary calls, empty
  collections, and ambiguous initializers; unit-level declarations rejected;
  changing a private body cannot change a public signature.
