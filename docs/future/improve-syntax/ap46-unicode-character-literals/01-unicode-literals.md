# AP46.1: Implement Unicode scalar literals

Package: [AP46](README.md). Status: planned; not implemented.

## Implementation and documentation

1. Replace the byte-range check and byte cast in
   `crates/fpas-lexer/src/lexer/strings.rs` with checked Unicode scalar decoding.
   Retain digit parsing and produce useful errors for surrogate, out-of-range
   and numerically oversized input without overflow or truncation.
2. Check adjacent quoted/code elements, token spans, diagnostic recovery and
   formatter serialization. A valid decoded scalar must survive formatting and
   reparsing, including supplementary characters and control codes.
3. Inspect highlighting, debugger expression lexing and editor diagnostics for
   duplicated byte-range assumptions. Reuse the shared lexer contract.
4. Update `docs/pascal/language/basics/primitive-types.md`, string character
   guidance, `docs/specs/grammar.ebnf`, affected diagnostic text and
   `docs/pascal/agent-reference.md`. Confirm authoring guidance remains accurate.
   Remove byte terminology for character-code literals; keep string indices
   explicitly defined as scalar indices.

## Verification

- Accept 0, 255, 256, 8364, the boundaries on either side of the surrogate gap,
  a supplementary-plane scalar and 1114111.
- Reject 55296, 57343, 1114112 and excessively long numeric input. Retain
  rejection of missing digits and invalid digit separators.
- Verify runtime equality with `string.Chr`, scalar Length/Ord/indexing and
  adjacent literal concatenation. Preserve the meaning of all old 0..255 codes.
- Cover formatter round trips and invalid-input diagnostic spans. Do not require
  preservation of the original numeric spelling if formatting normalizes it.
- Run lexer, formatter, relevant compiler/debugger tests, FPAS regressions,
  compact-reference examples and the applicable
  [required checks](../development-process.md#required-checks).

No repository-wide reformat or new string representation is part of this package.
