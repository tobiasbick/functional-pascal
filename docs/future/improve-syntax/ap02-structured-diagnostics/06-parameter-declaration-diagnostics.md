# AP02.6: Parameter declaration diagnostics

Package: [AP02: Structured diagnostics](README.md)

## Scope

Diagnose comma-separated or grouped parameter declarations with the canonical
semicolon-separated, individually typed form retained by
[AP08](../ap08-comma-separated-parameter-lists/README.md) (Q06).

## Prerequisites

- AP02.1 (code scheme).

## Implementation

- Recognize `function Add(A: integer, B: integer)` and
  `function Add(A, B: integer)` in the parser.
- Report a parser diagnostic that shows the canonical form, for example
  `function Add(A: integer; B: integer): integer;`, and recover at the
  closing parenthesis.

## Affected areas

- `crates/fpas-parser/src/parser/decl/routines.rs` (parameter lists).
- `crates/fpas-diagnostics/src/codes.rs`.

## Migration

None; the forms are already invalid.

## Documentation

Add the code to the diagnostics reference. `docs/pascal/language/functions/parameters.md`
may mention the diagnostic hint.

## Verification

- Parser tests for both invalid forms in functions, procedures, methods, and
  anonymous routines, and for the valid canonical form.
