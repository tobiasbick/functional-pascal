# AP02.2: Shared diagnostic record

Package: [AP02: Structured diagnostics](README.md)

## Scope

Extend the shared diagnostic record so every diagnostic carries code,
severity, phase, source file, start and end position, message, and optional
expected, found, and hint fields. Text and later JSON output render the same
record.

## Prerequisites

- AP02.1 (confirmed code scheme).

## Implementation

- Add optional end positions and expected/found details to the shared record.
  An unavailable location is absent, not a fabricated line zero.
- Define the column unit (Unicode scalar values) and resolve byte spans to
  one-based lines and columns with an exclusive end.
- Populate expected/found details from the parser's token expectations and the
  semantic type checker without parsing message text.
- Keep runtime diagnostics on the same record; mark synthetic runtime
  locations as points.
- The language service converts positions to the editor's encoding from the
  same record.

## Affected areas

- `crates/fpas-diagnostics/src/` (record, span, a focused source-range module).
- Parser expectation producer, sema type-mismatch producers, VM diagnostic
  mapping, `fpas-language-service` position conversion.

## Migration

Adapt all consumers of the record to optional positions. No FPAS source change.

## Documentation

Describe the record fields in the diagnostics page created in AP02.5, or add a
first version of `docs/pascal/tools/diagnostics.md` here if AP02.5 has not
landed yet.

## Verification

- Unit tests for Unicode ranges, missing positions, and expected/found details.
- Parser and sema tests assert structured details for representative errors.
- Language-service tests for Unicode position conversion.

## Reference

The reference branch `codex/syntax-changes` implemented this slice with
`fpas-diagnostics/src/source_range.rs` and `tests/structured_output.rs`.
Reuse its transport implementation with the confirmed Q01 code numbering.

## Result

The reference transport was reused with Q01 numbering. FileDiagnostic carries
the authoritative path; the shared span retains source identity and optional
positions. This slice ships with the remaining AP02 slices, as recorded in the
[implementation audit](implementation-audit.md).
