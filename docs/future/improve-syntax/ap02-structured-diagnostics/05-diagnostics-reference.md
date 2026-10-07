# AP02.5: Diagnostics reference

Package: [AP02: Structured diagnostics](README.md)

## Scope

Publish the diagnostics reference under `docs/pascal/tools/`, listing every
code with a wrong and a corrected example (Q01).

## Prerequisites

- AP02.1 (final code scheme). AP02.3 and AP02.4 for the project/build codes
  and the JSON envelope; codes added later are documented by their owning
  work package.

## Implementation

- Create or complete `docs/pascal/tools/diagnostics.md`: record fields,
  location rules, JSON envelope, and the code catalog.
- For each code, add a minimal wrong example and its correction.
- Add a test that every allocated code appears in the reference.

## Affected areas

- `docs/pascal/tools/diagnostics.md`, `docs/pascal/tools/README.md`.
- A documentation consistency test next to the code catalog tests.

## Migration

None.

## Documentation

This work package is the documentation.

## Verification

- The catalog test fails when a code is missing from the reference.
- Sample wrong examples produce the documented code.

## Result

At AP02 delivery, the reference covered all 120 allocated codes. Later
packages extend the same catalog and reference with their own allocations.
Compiler/linker invariant rows use invalid artifact or host-state examples
where no accepted FPAS source can
directly trigger the failure. Delivery is recorded in the
[implementation audit](implementation-audit.md).
