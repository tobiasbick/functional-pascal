# AP10.3: Remove record literals

Package: [AP10: Typed record construction](README.md)

## Scope

Remove the contextually typed `record ... end` literal and diagnose it with the
typed replacement.

## Prerequisites

- AP10.2 (no literals remain).

## Implementation

- Parser: remove `record_literal`; recognize the old form and report a
  diagnostic showing `TypeName(Field := Value)` with the expected type when
  known.
- Remove literal-specific sema and lowering paths.

## Affected areas

- `crates/fpas-parser/` expression parsing, `crates/fpas-sema/`,
  `crates/fpas-compiler/src/lowering/aggregates/records.rs`, `fpas-fmt`.

## Migration

None beyond AP10.2.

## Documentation

- `docs/specs/grammar.ebnf` (`record_literal`), `records.md`,
  `record-update.md`, diagnostics reference.

## Verification

- Rejection tests for the old literal with and without an expected type.
- Workspace tests and FPAS suite pass.
