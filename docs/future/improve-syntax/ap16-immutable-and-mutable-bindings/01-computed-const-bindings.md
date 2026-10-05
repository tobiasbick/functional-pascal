# AP16.1: Computed const bindings

Package: [AP16: Immutable and mutable bindings](README.md)

## Scope

Allow `const` bindings with computed initial values, while compile-time
constant contexts keep requiring compile-time constants. `var` and
`mutable var` are unchanged.

## Prerequisites

None.

## Implementation

- Sema: classify initializer expressions as compile-time constant or computed,
  separately from binding mutability. A `const` may have either; a
  compile-time context (case labels, array bounds, and later subrange bounds)
  requires a compile-time constant and names the non-constant part.
- Allow computed `const` at unit, program, routine, and statement level with
  scope-entry initialization in declaration order.
- Closures copy captured `const` values.
- Compiler and linker: computed `const` needs runtime initialization; keep
  compile-time constants folded and exported as before.

## Affected areas

- `crates/fpas-sema/src/check/decl/consts.rs`, constant evaluation,
  closure capture (`check/closures/capture.rs`, 436 lines at planning time;
  split traversal from capture collection if extending it).
- Compiler global initialization, `crates/fpas-linker/src/emit/constants.rs`,
  unit interfaces for exported constants.

## Migration

None.

## Documentation

- `docs/pascal/language/basics/constants.md`, `local-variables.md`,
  `docs/pascal/program-structure/initializing.md`, `docs/specs/grammar.ebnf`.

## Verification

- Computed `const` at every level; reassignment rejected; compile-time
  contexts reject computed values with the named part; captures copy;
  exported computed and compile-time constants across compiled units.
