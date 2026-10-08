# AP10.3: Remove record literals

Package: [AP10: Typed record construction](README.md)

## Scope

Remove the contextually typed `record ... end` literal and diagnose it with the
typed replacement.

## Prerequisites

- AP10.2 (no literals remain in positive consumers).

## Implementation

- Parser: remove `record_literal`; recognize the old form and report a
  diagnostic showing `TypeName(Field := Value)` with the expected type when
  known.
- Remove literal-specific sema and lowering paths.

## Affected areas

- `crates/fpas-parser/` expression parsing, `crates/fpas-sema/`,
  `crates/fpas-compiler/src/lowering/aggregates/records.rs`, `fpas-fmt`.

## Migration

Positive consumers are migrated by AP10.2. Negative fixtures that validate
record fields move to typed construction; fixtures for the removed syntax
assert the replacement diagnostic.

## Documentation

- `docs/specs/grammar.ebnf` (`record_literal`), `records.md`,
  `record-update.md`, diagnostics reference.

## Verification

- Rejection tests for the old literal with and without an expected type.
- Workspace tests and FPAS suite pass.

## Result

- The parser no longer has a record literal expression. `record ... end` in an
  expression reports FP2017 (`PARSE_REMOVED_RECORD_LITERAL`), skips the old
  field list, and keeps the following statements. The hint names the declared
  type when the literal is the whole initializer of a typed constant, variable,
  or field default (`Point(X := ..., Y := ...)`); elsewhere it uses
  `TypeName(...)`. The parser has no expected type in other positions.
- Sema no longer has anonymous record types: contextual literal annotation,
  structural record compatibility, and anonymous-record inference are removed.
  Record compatibility is nominal only.
- Lowering, formatter, closure discovery, source-id remapping, discard and
  task-bound analysis lost their literal paths. Record update lowering moved to
  `crates/fpas-compiler/src/lowering/aggregates/record_update.rs`.
- The debugger front end no longer lowers literals. The VM-internal structural
  `DebugExpression::Record` remains for its existing VM tests; replacing it
  with typed debugger construction is tracked in
  [compiler and language-limit follow-ups](../../compiler-panic-followups.md).

Coverage: `crates/fpas-parser/src/tests/errors/removed_record_literals.rs`
(with and without a declared type, qualified types, nested positions, empty
literal, recovery) and the FP2017 reference row in
`docs/pascal/tools/diagnostics.md`.
