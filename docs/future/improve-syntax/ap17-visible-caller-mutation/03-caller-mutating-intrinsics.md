# AP17.3: Caller-mutating intrinsics

Package: [AP17: Visible caller mutation](README.md)

## Scope

Make intrinsics that change a caller variable use explicit `var` arguments,
for example `Push(var Items, 3)`, and remove their special simple-variable rule.

## Prerequisites

- AP17.1 (`var` arguments).
- AP06.1 (decision on the dot form of mutating operations).

## Implementation

- Verify how `Push`, `Pop`, and other caller-mutating intrinsics mutate today.
- Declare their receiver parameter as `var` in the registry and generated
  declarations; reuse the AP17.1 checks instead of the special rule.
- Apply the AP06.1 decision for dot forms such as `Items.Push(3)`.

## Affected areas

- `crates/fpas-sema/src/std_registry/builtins/array/mutation.rs`,
  `crates/fpas-compiler/src/lowering/calls/arrays.rs`, generated
  `lib/api/Std/Arrays.fpas`, other mutating intrinsics found by the audit.

## Migration

Add `var` to every call of the affected intrinsics in all repository consumers.

## Documentation

- `docs/pascal/std/collections/array/mutating.md` and other affected `Std`
  pages; `fluent-calls.md` or its AP06 successor.

## Verification

- Direct, field, element, and forwarded `var` arguments; rejection of `const`
  and temporaries; dot forms per AP06.1; FPAS suite.
