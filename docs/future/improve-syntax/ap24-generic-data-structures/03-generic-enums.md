# AP24.3: Generic enums and recursion

Package: [AP24: Generic data structures](README.md)

## Scope

Allow user-defined generic enums with payloads, recursive generic types, and
nested matching on them.

## Prerequisites

- AP24.2 (generic records).
- AP20.2 (nested patterns).
- AP03.2 (closed-enum exhaustiveness rules).
- AP11.1 (finite recursive construction).

## Implementation

- Generic enum declarations and variant constructors with type-argument
  inference from arguments or the expected type.
- Recursive types (for example a generic list or tree) reuse AP11.1 finite
  construction checks after type substitution; reject mandatory stored-value
  cycles without a terminating construction.
- Constructor lookup and nested patterns over instantiated enums;
  exhaustiveness with AP03 rules.

## Affected areas

- Parser type definitions, sema generic types and pattern checking,
  `crates/fpas-compiler/src/lowering/case/`, `lowering/types/`,
  `fpas-unit` interfaces.

## Migration

None.

## Documentation

- `docs/pascal/language/types/generics.md`, `enums.md`, pattern-matching pages.

## Verification

- Multiple instantiations, recursion, invalid recursion, nested patterns,
  `Lookup.Missing` without context rejected, imported generic enums.
