# AP46.1: Implement Unicode scalar literals

Package: [AP46](README.md). Status: complete; implemented, verified, and independently reviewed.

## Implemented contract

Decimal `#N` literals denote one Unicode scalar in 0..1114111, excluding
55296..57343. Checked decoding rejects numeric overflow, surrogates and values
above the scalar domain with FP1005 and an actionable decimal example. Values
are never truncated, wrapped or replaced.

The existing decimal digit-separator and adjacent-element concatenation rules
remain unchanged. Every formerly valid 0..255 code keeps its original scalar
value. Literals remain compile-time strings, agree with `string.Chr(N)` and use
scalar indexing, Length and Ord. The runtime factory retains its checked dynamic
validation. No new character type, escape syntax or string representation is
introduced.

## Implementation ownership

- `crates/fpas-lexer/src/lexer/strings.rs` parses decimal text into u32 without
  overflow, then uses `char::from_u32` to validate and append the scalar. FP1005
  hints state the accepted domain, surrogate exclusion and valid examples. The
  shared digit scanner, source-position tracking and recovery paths are retained.
- `crates/fpas-fmt/src/emit/expr/literal.rs` retains its scalar-based serializer:
  controls use decimal codes and printable scalars use quoted strings. Formatting
  may normalize numeric spelling while preserving values and idempotence.
- Debugger expression parsing and editor analysis reuse the parser and shared
  lexer. The existing VS Code numeric-character pattern has no byte-range limit;
  no duplicate decoding or highlighting change is required.
- `crates/fpas-build/build/compiler_identity.rs` includes lexer sources in the
  compiler fingerprint, so source-matching sidecars from a different compiler
  identity are rebuilt. No artifact version change or source migration is needed.
- `docs/pascal/language/basics/primitive-types.md`,
  `docs/pascal/language/types/string/format-chars.md`, `docs/specs/grammar.ebnf`
  and `docs/pascal/agent-reference.md` describe the same Unicode scalar domain.
  Existing repository authoring-skill guidance remains accurate.

## Regression ownership

- `crates/fpas-lexer/src/tests/strings.rs` owns accepted boundaries, separator
  spellings, all formerly valid 0..255 values, adjacent Unicode elements and
  original-source byte offsets, scalar columns and source identifiers.
- `crates/fpas-lexer/src/tests/errors/strings.rs` owns surrogate boundaries,
  out-of-domain values, u32 overflow and oversized text, precise diagnostic
  spans, actionable hints, recovery without forged concatenation, missing
  digits and invalid separators.
- `crates/fpas-fmt/tests/literal_regressions.rs` verifies decoded values after
  serialization and reparsing, including controls, apostrophes and
  supplementary scalars, with idempotence checks.
- `crates/fpas-compiler/src/tests/computed_const/character_literals.rs` executes
  runtime-factory equality, Length, Ord, scalar indexing and constant case labels.
  Embedded numeric source preserves the literal forms through repository
  formatting; no duplicate FPAS fixture is needed.
- `crates/fpas-debug/tests/unicode_character_literals.rs` exercises watch
  decoding, concatenation and indexing, plus located invalid-scalar errors
  through the JSONL evaluation boundary.
- `crates/fpas-language-service/tests/unicode_character_literals.rs` verifies
  diagnostics appear and clear across unsaved edits without changing disk source.
- Existing FPAS string regressions retain dynamic factory errors and scalar
  operations. The CLI compact-reference test checks and executes the updated
  `ReferenceBasics` example; the handbook character examples are also verified.

## Verification and open follow-ups

Rust formatting, workspace build and workspace tests including doc-tests pass
with the two known baseline TCP cases excluded. FPAS regressions, source
formatting, app/example project checks, handbook and compact-reference examples,
documentation links, editor grammar checks and release distribution checks pass.
Independent review found no blocking issues.

The excluded tests are
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` and
`tcp_backpressure_can_time_out_and_cancel_without_closing`. Their implementations
and tests are unchanged by AP46; baseline evidence is retained in
[AP41's verification notes](../ap41-canonical-qualified-names/02-qualified-name-rules.md).
No selected AP46 work remains open.
