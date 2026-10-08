# AP14.1: Migrate properties to methods

Package: [AP14: Remove computed properties](README.md)

## Scope

Replace every property declaration and use with getter functions and setter
procedures while properties are still accepted.

## Prerequisites

- The AP06 decision that record methods remain dot-callable (recorded).

## Implementation

- Remove each property declaration. Its existing `read` and `write` methods
  stay under their declared names; no accessor is generated or renamed.
- Replace property reads with calls of the `read` method (`S.Zoom` becomes
  `S.GetZoom()`) and property assignments with calls of the `write` method
  (`C.Value := 20` becomes `C.SetBase(20)`).
- When a public property had a private accessor, make that accessor `public`
  explicitly in the migration.

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

## Result

- The four `Zoom` properties in `examples/math/explorer/Camera.fpas`,
  `examples/math/julia/julia.fpas`, `examples/math/mandelbrot/mandelbrot_model.fpas`,
  and `examples/math/newton/newton.fpas` were removed; callers use
  `GetZoom()` (`Explorer.fpas`, `mandelbrot_view.fpas`,
  `tests/apps/fractal_camera_test.fpas`). The accessors kept their names and
  visibility.
- `examples/pascal/record-methods/properties.fpas` became `accessors.fpas` and
  calls `GetBase()` and `SetBase(...)` directly; its output is unchanged.
- The formatter golden `record_visibility` dropped its property; the formerly
  private `read` method `Hidden` is now `public`, as the migration rule requires.
- No library, app, or `Std` API used properties.
