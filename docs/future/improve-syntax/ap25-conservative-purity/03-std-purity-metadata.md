# AP25.3: Standard-library purity metadata

Package: [AP25: Conservative purity](README.md)

## Scope

Mark standard-library routines as pure where their implementation is verified
to satisfy the AP25 contract, so pure functions can call them.

## Prerequisites

- AP25.2 (pure function checker).

## Implementation

- Audit intrinsic and `Std` source implementations; never infer purity from
  names.
- Record purity in the registry and generated declarations; source-defined
  `Std` units use `pure function` and are checked like user code.

## Affected areas

- `crates/fpas-sema/src/std_registry/`, `crates/fpas-std/src/`, `lib/Std/`,
  `lib/api/Std/`.

## Migration

None.

## Documentation

- Mark pure routines on the `Std` pages.

## Verification

- Pure calls into marked routines accepted; unmarked and side-effecting
  routines rejected; a test that every marked intrinsic is covered by the audit.
