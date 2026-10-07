# AP22.2: Inference from typed construction

Package: [AP22: Limited local inference](README.md)

## Scope

Allow local bindings without a type annotation when the initializer is a typed
record construction, for example `const P := Point(X := 1, Y := 2);`.

## Prerequisites

- AP22.1 (local inference rules).
- AP10.1 (typed construction).
- The open decision on enum variant constructors in the [package README](README.md).

## Implementation

- Sema: infer the record type from the construction's type name. Generic
  constructions whose type arguments are not determined by their fields are
  underconstrained and require an annotation.
- Apply the recorded decision on enum variant constructors.

## Affected areas

- `crates/fpas-sema/src/check/decl/vars.rs`, `check/decl/consts/`.

## Migration

None.

## Documentation

- `local-variables.md`, `records.md`.

## Verification

- Inferred record construction, imported record types, underconstrained
  generic constructions rejected, variant constructors as decided.
