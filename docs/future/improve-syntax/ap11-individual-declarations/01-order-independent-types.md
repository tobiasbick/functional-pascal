# AP11.1: Order-independent type declarations

Package: [AP11: Individual declarations](README.md)

## Scope

Resolve type references across all type declarations of a unit or program
regardless of order, including mutually recursive types (Q07). Constants and
variables keep declaration order.

## Prerequisites

None.

## Implementation

- Sema: collect all type headers of a compilation unit before checking
  signatures and type bodies.
- Reject alias cycles and infinite-layout cycles with a diagnostic naming the
  cycle.
- Keep value initialization order unchanged; do not reorder effects.
- Preserve scope and visibility of exported types across compiled units.

## Affected areas

- `crates/fpas-sema/src/check/entry.rs`, `check/decl/types/`.
- Unit interface export of recursive types.

## Migration

None; existing order-dependent sources stay valid.

## Documentation

- `docs/pascal/language/types/README.md` or the records/enums pages: types may
  reference later and mutually recursive types.

## Verification

- Forward references, mutually recursive records and enums, alias cycles,
  imported recursive types, and unchanged constant/variable ordering errors.
