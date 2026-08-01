# `fpas-lexer` review follow-up

Classification: lexical correctness, recovery, and public span API. The current language rule remains ASCII-only identifiers with an optional leading BOM.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| LEXER-01 | P2 | `crates/fpas-lexer/src/lexer/trivia.rs:27` | Every U+FEFF is treated as whitespace, although only a leading BOM is allowed. Invisible mid-source BOMs split tokens without a diagnostic. | Consume BOM only once at offset zero; later U+FEFF follows the unexpected-character path. | BOM at start, middle, and EOF; confirm offset/line/column after leading BOM. |
| LEXER-02 | P2 | `crates/fpas-lexer/src/lexer/identifiers.rs:24` | Recovery after a non-ASCII identifier character consumes only the non-ASCII run. `fooébar` reports an error but leaves `bar` as a valid identifier token. | After an invalid identifier character, consume the complete identifier-like remainder through ASCII letters, digits, and underscore until a real delimiter. | `fooébar`, multiple invalid runs, and a following genuinely separate token. |
| LEXER-03 | P2 | `crates/fpas-lexer/src/comments.rs:43`, `src/lexer/navigation.rs:54` | `SourceComment::is_end_of_line` recognizes only LF, while lexer locations also treat standalone CR as a line ending. | Find line start using both CR and LF, handling CRLF as one logical break. | LF, CRLF, and CR-only comments on their own line and after code. |
| LEXER-04 | P2 | `crates/fpas-lexer/src/comments.rs:30,36` | Public comment helpers slice unchecked public spans and can panic for another source, overflow, or non-character boundary. | Use checked range construction and `str::get`, returning `Option`/`Result`; document source identity and span invariants. | Short/wrong source, overflow, invalid UTF-8 boundary, and valid roundtrip. |
| LEXER-05 | P3 | `crates/fpas-lexer/src/lexer/identifiers.rs:13` | Every identifier/keyword is built character-by-character into a `String`, even when a keyword immediately discards it. Impact is not measured. | Scan byte boundaries, inspect a borrowed slice for keyword recognition, and allocate only real identifier payloads. | Preserve tokens/spans; benchmark before recording a performance claim. |
| LEXER-06 | P3 | `crates/fpas-lexer/src/lib.rs:1`, `src/span.rs:1` | Public crate, token, span, and comment contracts are incompletely documented. | Add crate/module docs and define byte offsets, lengths, line endings, one-based positions, and error-token progression. | Rustdoc examples or API tests for span extraction and recovery. |

## Implementation notes

LEXER-01 is a conformance correction to the already documented grammar, not a language change. Confirm parser/formatter consumers of CR-only and invalid-identifier recovery before changing token sequences.
