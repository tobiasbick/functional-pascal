# fpas-lexer Review

## Summary

No high-confidence tokenizer correctness issue was found. The crate passed package Clippy.

## Findings

### Low: Core public lexer entry points lack rustdoc

Evidence: `crates/fpas-lexer/src/lib.rs:21` defines public `SpannedToken`, and `crates/fpas-lexer/src/lib.rs:26` defines public `lex`, both without item-level docs.

Impact: `lex_with_comments`, `collect_comments`, and `lex_with_source_id` are documented, but the primary simple lexer path and token wrapper are not. This leaves the source-id and EOF-token expectations implicit for downstream parser and formatter users.

Suggested fix: Document `SpannedToken` fields and `lex` behavior, especially whether `Token::Eof` is included and how spans are assigned.

## Verification

- `cargo clippy -p fpas-lexer --all-targets -- -D warnings` passed.

