# AP09.2: Named arguments for enum variant constructors

Package: [AP09: Named arguments](README.md)

## Scope

Allow fully named arguments for enum variants with data, using the declared
field names, for example `Shape.Rectangle(Width := 3.0, Height := 4.0)`.
Patterns stay positional.

## Prerequisites

- AP09.1 (named argument mapping).

## Implementation

- Reuse the AP09.1 mapping for variant constructors, including generic enums
  and imported variants.
- Diagnostics for unknown, duplicate, missing, and mixed arguments list the
  variant's field names.

## Affected areas

- `crates/fpas-sema/src/check/` (variant construction), compiler variant
  construction lowering.

## Migration

None.

## Documentation

- `docs/pascal/language/types/enums.md`.

## Verification

- Positional and named variant construction, reordered fields with
  side-effect traces, imported and qualified variants, rejections.
- Patterns remain positional.
