# AP24.2: Generic records

Package: [AP24: Generic data structures](README.md)

Status: complete.

## Scope

Allow user-defined generic records, for example
`type Pair of (K: Comparable, V) = record ... end record;`, with typed
construction and constructor type-argument inference.

AP10.1 supplies construction for concrete record types and their aliases.
This work package introduces generic record declarations and extends that
construction with generic type substitution and type-argument inference.

## Agreed rules

- Use `Pair(Key := ..., Value := ...)` with AP10's named-field construction.
  Explicit constructor type applications such as
  `Pair of (integer, string)(...)` are rejected; put the type arguments in a
  type declaration or annotation.
- Follow the package's [constructor inference rules](README.md#constructor-type-inference):
  consider all supplied fields together, use the expected type to fill missing
  parameters, and reject conflicts or parameters left undetermined. Do not
  search for a common type or promote numeric types to solve a conflict.
- Follow the package's [default rules](README.md#generic-record-defaults):
  defaults must be valid under the declared constraints and never contribute
  to inference. Preserve AP10's visibility, declaration scope, and evaluation
  order.

## Prerequisites

- AP24.1 (parenthesized type arguments).
- AP10.1 (typed construction).

## Implementation

- The parser accepts record parameters after `of` and type applications in
  annotations. Explicit constructor applications receive an annotation hint.
- Shared substitution resolves fields, nested containers, and callable
  signatures. Constructor inference combines supplied fields and the expected
  type without using defaults. Constraints, nominal arguments, visibility,
  generic defaults, and recursive parameter forwarding are checked.
- Instance-method inference retains receiver arguments and keeps each method's
  own parameters distinct from enclosing parameters. Callable fields receive
  their expected signatures after joint inference.
- The compiler uses a shared erased layout with typed field access, updates,
  callable fields, and postfix results. Unit interfaces retain generic
  parameters and applications; independently compiled units link through the
  existing record layout model.
- Hover and completion display instantiated field types for local and imported
  records, including nested fields and native member chains.

## Affected areas

- `crates/fpas-parser/src/parser/decl/` and `crates/fpas-fmt/src/emit/`:
  record headers and applications.
- `crates/fpas-sema/src/types/`, `check/decl/types/`,
  `check/expr/record_construction/`, and `check/calls/`: substitution,
  declaration checks, joint inference, and method parameter scopes.
- `crates/fpas-sema/src/check/expr/construction_inference.rs`: joint field
  inference shared with enum constructors.
- `crates/fpas-compiler/src/lowering/aggregates/`, `context/value_conversion.rs`,
  `routines.rs`, and `types.rs`: erased storage and concrete access types.
- `crates/fpas-unit/src/interface/` and `crates/fpas-sema/src/interface/`:
  generic declarations and nominal application references.
- `crates/fpas-language-service/src/intellisense/` and `navigation/resolve.rs`:
  instantiated fields and member resolution.

## Migration

None.

## Documentation

- `docs/pascal/language/types/generics.md`, `records.md`, `record-methods.md`,
  and the types overview.
- `docs/specs/grammar.ebnf`, `docs/pascal/tools/fmt-style.md`, and
  `editor-integration.md`.

## Verification

- Several concrete instantiations, nesting, constraints, invalid type
  arguments, underconstrained construction, imported generic records,
  equality.
- Inference from supplied fields, completion of partially determined
  parameters by the expected type, and identical inference after reordering
  named fields; conflicts between fields or with the expected type rejected.
- Explicit constructor type applications rejected with a type-annotation
  hint; ordinary construction with an annotation or type alias accepted.
- Defaults valid under the declared constraints accepted, invalid generic
  defaults rejected even when one instantiation would accept them, and
  omitted defaults leaving type parameters undetermined.
- Recursive and mutually recursive records retain ordered parameters;
  transformed arguments and mandatory stored-value cycles are rejected.
- Method receiver arguments remain fixed when caller and method parameters
  share names. Generic callbacks and callable fields execute after inference,
  including constructor field reordering and chained generic method results.
- Fresh and reused unit sidecars preserve arguments, aliases, defaults,
  methods, nested instances, recursive fields, and private construction rules.
- Language-service tests cover concrete field hover, completion, imported
  nesting, and native string members. The runnable example and
  `tests/runner/generic_records_test.fpas` cover the real runner.
- Workspace and FPAS suites, source formatting, documentation examples and
  links, example/app project checks, and the real Extension Host verify the
  integrated implementation.
