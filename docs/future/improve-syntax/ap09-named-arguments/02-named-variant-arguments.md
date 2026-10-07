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

## Result

Delivered for the current, non-generic enums, including short, qualified, and
imported variants across compiled units. Agreed scope details:

- `Ok(…)`, `Error(…)`, and `Some(…)` have no declared field names and stay
  positional (FP3026).
- Enum patterns stay positional; a named pattern field is FP3026.
- Generic enums do not exist yet; their named construction is recorded as a
  requirement in [AP24.3](../ap24-generic-data-structures/03-generic-enums.md).
- Editor: variant fields are document symbols, so navigation, signature help,
  and rename cover `Field := Value` labels. Renaming any field previously
  failed with a false conflict; that check now skips owner-bound symbols
  (fields, properties, events). Rename still misses `Field := Value` labels in
  anonymous record literals; recorded in
  [AP10.1](../ap10-typed-record-construction/01-typed-construction.md).
- Standard-library enum variants whose names are FPAS keywords are added from
  the registry without field symbols; their labels are not navigable.
