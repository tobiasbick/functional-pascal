# AP08: Comma-separated parameter lists

Status: rejected and closed by decision Q06. No parameter-separator migration
is planned, and this package has no work packages. Completion is tracked in the
[central README](../README.md).

## Decisions (Q06)

- Keep `;` between declared parameters and `,` between call arguments.
- Each parameter has its own type annotation; grouped declarations such as
  `A, B: integer` remain invalid.
- Diagnostics for comma separators and grouped declarations show the canonical
  form, for example `function Add(A: integer; B: integer): integer;`. These
  diagnostics and their regression coverage are tracked in
  [AP02.6](../ap02-structured-diagnostics/06-parameter-declaration-diagnostics.md).
