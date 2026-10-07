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
- Preserve the existing constant-expression forms with compile-time-known
  operands. Treat function and method calls as computed, including user,
  standard-library/intrinsic, and native type-operation calls; apparent or
  inferred purity does not make a call a compile-time constant. Do not execute
  routines at compile time to classify their results.
- Propagate computed classification through references to computed `const`
  bindings, including exported bindings imported from compiled units. An
  initializer's literal operands or an optimization must not hide its
  non-constant dependency. Diagnostics identify the offending call or
  computed binding when a compile-time context rejects it.
- Allow computed `const` at unit and program level and in routine statement
  lists, including nested scopes and control-flow bodies. Local initialization
  happens once whenever execution reaches the declaration, in statement
  order; do not move it to scope entry. A loop-body declaration initializes
  on each iteration that reaches it, and untaken branches do not initialize
  their bindings. Preserve existing program and unit initialization order.
- A computed binding placed before a loop evaluates once for that execution
  of the enclosing statement list. Optimizations must preserve observable
  effects, failures, and reachability, independently of whether a binding is
  immutable or classified as computed.
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
- Existing constant-expression forms remain accepted. Calls stay computed
  even for routines that return a literal; arithmetic on a computed `const`
  stays computed. Cover transitive dependencies and imported computed
  constants, and verify that constant-context diagnostics name the call or
  binding responsible.
- During implementation, verify initializer call counts and ordering relative
  to surrounding statements, repeated loop iterations, skipped branches,
  early exits, and initializer failures. Compare a declaration inside a loop
  with one placed before it. No separate cloud-environment preflight is
  required; these are compiler/runtime regression tests.
