# `fpas-parser` review follow-up

Classification: parser correctness and recovery. PARSER-01 enforces the existing program grammar. Any newly chosen recovery semantics must not change accepted valid programs.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| PARSER-01 | P1 | `crates/fpas-parser/src/parser/program.rs:45,59` | After parsing `end.`, the program parser returns without checking EOF. Arbitrary identifiers or a second program are silently ignored. | Apply the Unit parser's trailing-input check after the final period, with a focused diagnostic and synchronization. | Identifier, second program, and other tokens after `end.` each produce a trailing-input diagnostic. |
| PARSER-02 | P2 | `crates/fpas-parser/src/parser/stmt/mod.rs:12` | A missing semicolon stops the complete statement list, loses later statements from the partial AST, and produces misleading outer diagnostics. | Continue until strong boundary tokens; diagnose a missing separator once and recover at a statement start or boundary. | Missing separator in blocks, if branches, repeat, and case else while retaining the following statement. |
| PARSER-03 | P3 | `crates/fpas-parser/src/lib.rs:41,86`, `src/ast/program.rs:5` | Public parse entry points, diagnostic ordering, partial-AST behavior, and many AST/span contracts lack Rustdoc. | Add crate/API documentation and adopt missing-doc enforcement incrementally. | External API or Rustdoc tests for successful and partial parses. |
| PARSER-04 | P3 | `crates/fpas-parser/src/parser/core.rs:221` | Lower confidence: synthetic EOF adds span length directly to column, which may be wrong for byte lengths after Unicode or multiline tokens. | Confirm lexer span semantics, then compute EOF from the source/last token using line-aware logic. | Exact EOF line/column after Unicode and every supported line ending. |

## Implementation notes

Production modules are currently focused and under the project size threshold; no structural split is required. PARSER-02 should be driven by recovery tests before changing the loop.
