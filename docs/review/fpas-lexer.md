# `fpas-lexer` review follow-up

Classification: lexical correctness, recovery, and public span API. The current language rule remains ASCII-only identifiers with an optional leading BOM.
Status: all findings completed 2026-08-03.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| LEXER-01 | P2 | `crates/fpas-lexer/src/lexer/trivia.rs` | **Done.** Every U+FEFF was treated as whitespace, although only a leading BOM is allowed. Invisible mid-source BOMs split tokens without a diagnostic. | Consume BOM only once at offset zero; later U+FEFF follows the unexpected-character path. | BOM at start, middle, and EOF; confirm offset/line/column after leading BOM. |
| LEXER-02 | P2 | `crates/fpas-lexer/src/lexer/identifiers.rs` | **Done.** Recovery after a non-ASCII identifier character consumed only the non-ASCII run. `fooébar` reported an error but left `bar` as a valid identifier token. | After an invalid identifier character, consume the complete identifier-like remainder through ASCII letters, digits, and underscore until a real delimiter. Apply the same recovery to a non-ASCII initial letter. | `fooébar`, multiple invalid runs, a non-ASCII initial letter, and a following genuinely separate token. |
| LEXER-03 | P2 | `crates/fpas-lexer/src/comments.rs` | **Done.** `SourceComment::is_end_of_line` recognized only LF, while lexer locations also treat standalone CR as a line ending. | Find line start using both CR and LF, handling CRLF as one logical break. Reuse this implementation from the formatter. | LF, CRLF, and CR-only comments on their own line and after code. |
| LEXER-04 | P2 | `crates/fpas-lexer/src/comments.rs`, `src/span.rs` | **Done.** Public comment helpers sliced unchecked public spans and could panic for another source, overflow, or non-character boundary. | Use checked range construction and `str::get`, returning `Option`; document source identity and span invariants. | Short/wrong source, overflow, invalid UTF-8 boundary, and valid roundtrip. |
| LEXER-05 | P3 | `crates/fpas-lexer/src/lexer/identifiers.rs`, `src/token/keywords.rs` | **Done.** Every identifier/keyword was built character-by-character into a `String`, even when a keyword immediately discarded it. | Scan byte boundaries, inspect a borrowed slice for keyword recognition, and allocate only real identifier payloads. Do not record an unmeasured speedup. | Preserve tokens/spans; explain measurement limits before recording any performance claim. |
| LEXER-06 | P3 | `crates/fpas-lexer/src/lib.rs`, `src/span.rs`, `src/comments.rs` | **Done.** Public crate, recovery, EOF, source identity, span, and comment contracts were incomplete. | Document byte offsets, lengths, line endings, one-based positions, error-token progression, and safe extraction. | Rustdoc example plus API tests for span extraction and recovery. |

## Implementation notes

LEXER-01 is a conformance correction to the already documented grammar, not a language change. Confirm parser/formatter consumers of CR-only and invalid-identifier recovery before changing token sequences.

## Implementation record

- Only an initial `U+FEFF` is trivia. Later BOMs use the existing unexpected-character diagnostic
  and retain accurate UTF-8 byte spans.
- Invalid ASCII/non-ASCII identifier-like sequences are consumed through the next real delimiter,
  including when the first character is non-ASCII. Recovery emits one diagnostic and no partial
  identifier token.
- Identifier scanning now borrows the exact source slice for keyword lookup and allocates only an
  actual `Token::Ident` payload. The FPAS benchmark harness times already compiled program runtime,
  not lexer/frontend work, so no speedup or benchmark-history entry is claimed.
- `Span` and `SourceComment` extraction use checked addition and `str::get`. Manually invalid,
  overflowing, out-of-source, or mid-codepoint spans return `None` instead of panicking.
- Comment line ownership recognizes `LF`, `CRLF`, and bare `CR`; `fpas-fmt` now reuses the lexer API
  rather than maintaining a second implementation.
- Crate/API documentation now defines byte and position units, source identity, recovery, and the
  guaranteed EOF token. No FPAS syntax or semantics changed.
- User documentation: `docs/pascal/language/basics/number-literals.md` and `comments.md`.

## Verification

- `cargo test -p fpas-lexer`: 189 unit tests and 1 doctest passed.
- `cargo test -p fpas-fmt`: passed, including comment and CR-only regressions.
- `cargo doc -p fpas-lexer --no-deps`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo build --workspace`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- No `.fpas` source changed, so no targeted FPAS formatting or suite run was required.
- No benchmark history was recorded because the current harness does not measure lexer/frontend
  time and no runtime speedup is claimed.
