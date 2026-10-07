# AP16.1: Computed const bindings

Package: [AP16: Immutable and mutable bindings](README.md)

## Scope

Allow `const` bindings with computed initial values, while compile-time
constant contexts keep requiring compile-time constants. At AP16.1 delivery,
`var` and `mutable var` were unchanged; AP16.3 completed their keyword switch.

## Prerequisites

None.

## Implementation

- Sema: classify initializer expressions as compile-time constant or computed,
  separately from binding mutability. A `const` may have either; a
  compile-time context (case labels and later subrange bounds)
  requires a compile-time constant and names the non-constant part.
- Enforce compile-time constants for scalar `case` value labels and both
  endpoints of a range, including directly written expressions. Reject runtime
  calls and computed bindings there; use guards for dynamic conditions.
  Preserve enum, `Option`, and `Result` patterns. Migrate existing tests that
  evaluate runtime labels to guards and add rejection tests for the old forms.
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

- Parser statement bindings and recovery, formatter and AST traversals.
- `crates/fpas-sema/src/check/decl/consts/`, shared initializer checking,
  and closure capture under `check/closures/capture/` (capture collection and
  AST traversal are split).
- Compiler local/global initialization and interface-backed imports, plus
  `crates/fpas-unit/src/interface/` for exported constant classification.

## Migration

Runtime case labels and ranges are expressed with guard conditions. Binding
keyword migration remains in AP16.2 and AP16.3.

## Documentation

- `docs/pascal/language/basics/constants.md`, `local-variables.md`,
  `docs/pascal/language/pattern-matching/`,
  `docs/pascal/program-structure/units.md`, `docs/specs/grammar.ebnf`.

## Verification

- Computed `const` at every level; reassignment rejected; compile-time
  contexts reject computed values with the named part; captures copy;
  exported computed and compile-time constants across compiled units.
- Existing constant-expression forms remain accepted. Calls stay computed
  even for routines that return a literal; arithmetic on a computed `const`
  stays computed. Cover transitive dependencies and imported computed
  constants, and verify that constant-context diagnostics name the call or
  binding responsible.
- Static case labels and ranges are accepted; runtime calls, variable references,
  computed constants, and their dependent expressions are rejected in labels
  and either range endpoint. Guards still evaluate dynamic expressions.
- During implementation, verify initializer call counts and ordering relative
  to surrounding statements, repeated loop iterations, skipped branches,
  early exits, and initializer failures. Compare a declaration inside a loop
  with one placed before it. No separate cloud-environment preflight is
  required; these are compiler/runtime regression tests.

## Result

The existing global-initialization and linker paths already support runtime
initializers. Computed and non-scalar constants use immutable global imports;
no constant-pool or VM instruction change is needed. AP16.1 introduced
compiled-unit format 7 for static/computed classification; AP16.3 advances
it to version 8 for the binding and parameter switch. Older sidecars rebuild.

Arrays currently have dynamic sizes; there is no bounded-array constant context
to change. Subrange bounds join the constant classification in AP18.
