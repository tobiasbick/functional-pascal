# `fpas-parser` review follow-up

Classification: parser correctness and recovery. PARSER-01 enforces the existing program grammar. Any newly chosen recovery semantics must not change accepted valid programs.
Status: PARSER-01 through PARSER-04 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| PARSER-01 | P1 | `crates/fpas-parser/src/parser/program.rs:45,59` | After parsing `end.`, the program parser returns without checking EOF. Arbitrary identifiers or a second program are silently ignored. | Apply the Unit parser's trailing-input check after the final period, with a focused diagnostic and synchronization. | Identifier, second program, and other tokens after `end.` each produce a trailing-input diagnostic. |
| PARSER-02 | P2 | `crates/fpas-parser/src/parser/stmt/mod.rs:12` | A missing semicolon stops the complete statement list, loses later statements from the partial AST, and produces misleading outer diagnostics. | Continue until strong boundary tokens; diagnose a missing separator once and recover at a statement start or boundary. | Missing separator in blocks, if branches, repeat, and case else while retaining the following statement. |
| PARSER-03 | P3 | `crates/fpas-parser/src/lib.rs:41,86`, `src/ast/program.rs:5` | Public parse entry points, diagnostic ordering, partial-AST behavior, and many AST/span contracts lack Rustdoc. | Add crate/API documentation and adopt missing-doc enforcement incrementally. | External API or Rustdoc tests for successful and partial parses. |
| PARSER-04 | P3 | `crates/fpas-parser/src/parser/core.rs:221` | Lower confidence: synthetic EOF adds span length directly to column, which may be wrong for byte lengths after Unicode or multiline tokens. | Confirm lexer span semantics, then compute EOF from the source/last token using line-aware logic. | Exact EOF line/column after Unicode and every supported line ending. |

## Implementation notes

Production modules are currently focused and under the project size threshold; no structural split is required. PARSER-02 should be driven by recovery tests before changing the loop.

## Implementation record

- PARSER-01 now requires EOF after a successfully consumed program terminator. One focused
  diagnostic identifies the first trailing token, skips the remainder, and keeps the program AST
  span limited to the valid program. A missing final `.` retains its existing single diagnostic.
- PARSER-02 recognizes valid statement-list boundaries without requiring `;`. When another token
  follows a statement, it emits one separator diagnostic and synchronizes at a semicolon, strong
  boundary, or statement start. Recovery resumes the ordinary statement parser after a discovered
  semicolon, preventing both AST loss and separator-diagnostic cascades.
- PARSER-03 documents the crate, parse entry points, diagnostic ordering, partial-AST behavior, and
  every exported AST contract. `#![deny(missing_docs)]` makes the coverage enforceable; successful
  and partial parses are executable Rustdoc examples.
- PARSER-04 adds an exact lexer-captured end position to each `SpannedToken`. Synthetic EOF uses
  that position and preserves `source_id`, so Unicode and LF, CRLF, or CR inside the last token no
  longer corrupt EOF coordinates. The public API states that removed trailing trivia cannot be
  reconstructed without the original EOF token.
- FPAS syntax and semantics are unchanged. Normative `docs/pascal/` pages remain unchanged because
  the accepted valid language is unchanged; only invalid-input diagnostics, recovery, and Rust API
  contracts changed.

## Verification

- Baseline: `cargo test -p fpas-parser --locked` — passed: 232 tests plus doc tests.
- Baseline: `cargo doc -p fpas-parser --no-deps --locked` with `-D missing_docs` — failed on the
  undocumented crate, entry points, diagnostics, and exported AST, confirming PARSER-03.
- `cargo test -p fpas-lexer --locked` — passed: 177 tests plus doc tests.
- `cargo test -p fpas-parser --locked` — passed: 244 tests plus 2 doc tests.
- `cargo rustdoc -p fpas-parser --lib --locked -- -D missing_docs -D
  rustdoc::broken_intra_doc_links` — passed.
- `cargo clippy -p fpas-lexer --all-targets --locked -- -D warnings` — passed.
- `cargo clippy -p fpas-parser --all-targets --locked -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked --quiet` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- Independent final audit found no remaining code, recovery, API-contract, or test-coverage blocker.
