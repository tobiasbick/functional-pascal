# AP17.3: Caller-mutating intrinsics

Package: [AP17: Visible caller mutation](README.md)

## Scope

Make intrinsics that change a caller variable use explicit `var` arguments,
including the receiver marking for native array `Push` and `Pop` agreed in
AP06.1, and remove their special simple-variable rule. These remain native
type operations, without a parallel free-call API.

## Prerequisites

- AP17.1 (`var` arguments).
- AP06.1 (decision on explicit receiver mutation marking).

Coordinate with AP06.3's removal of the old type-helper units. Record the
concrete implementation order when the receiver-marking syntax is settled;
the migration must not reintroduce a public `Std.Arrays` API.

## Implementation

- Verify how `Push`, `Pop`, and other caller-mutating intrinsics mutate today.
- Declare their receiver parameter as `var` in the registry and catalog-derived
  editor signatures; reuse the AP17.1 checks instead of the special rule.
- Apply the AP06.1 decision for receiver marking in the one native call form.

## Affected areas

- `crates/fpas-sema/src/std_registry/builtins/array/mutation.rs`,
  `crates/fpas-compiler/src/lowering/calls/arrays.rs`, generated
  native operation signatures replacing `lib/api/Std/Arrays.fpas`, and other
  mutating intrinsics found by the audit.

## Migration

Apply the agreed explicit receiver marking to every affected native operation
call in all repository consumers. Coordinate remaining ordinary-call and
import removal with AP06.3.

## Documentation

- Array operation/mutation documentation replacing the former
  `docs/pascal/std/collections/array/mutating.md` unit API, other affected
  pages, and `fluent-calls.md` or its AP06 successor.

## Verification

- Direct, field, element, and forwarded `var` arguments; rejection of `const`
  and temporaries; dot forms per AP06.1; FPAS suite.
