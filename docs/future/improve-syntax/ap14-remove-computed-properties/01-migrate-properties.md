# AP14.1: Migrate properties to methods

Package: [AP14: Remove computed properties](README.md)

## Scope

Replace every property declaration and use with getter functions and setter
procedures while properties are still accepted.

## Prerequisites

- The AP06 decision that record methods remain dot-callable (recorded).

## Implementation

- For each property: its read accessor becomes a public instance function
  called with parentheses; its write accessor becomes an instance procedure.
- Replace property reads with method calls and property assignments with
  setter calls; keep visibility.

## Affected areas

- `examples/pascal/record-methods/properties.fpas`, `examples/math/explorer/Camera.fpas`,
  `examples/math/mandelbrot/mandelbrot_model.fpas`, `examples/math/newton/newton.fpas`,
  `examples/math/julia/julia.fpas` (planning-time inventory; recheck).
- The formatter golden file that contains properties (convert it to the
  method form and keep a separate negative test for AP14.2).

## Migration

This work package is the migration.

## Documentation

Documentation examples that use properties outside the property page.

## Verification

- No property use remains outside `record-properties.md` and negative tests.
- Migrated examples check and run with unchanged output.
