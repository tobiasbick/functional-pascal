# Task 10 — Keep the routine AST when recovering invalid top-level `static`

Status: in progress
Severity: P2
Difficulty: easy
Language gate: no
Depends on: none

## Goal

`program T; static function Foo(): integer; begin return 1 end; begin end.` still has `Foo` in `declarations` after reporting `PARSE_INVALID_STATIC_PLACEMENT`.

## Bug

`crates/fpas-parser/src/parser/decl/mod.rs` `recover_invalid_static_decl` parses the function/procedure then **discards** it (`let _ = self.parse_function_decl(...)`). Later stages never see `Foo`. Body errors inside Foo can still be recorded, which is good — keep that.

## Fix

Push the parsed routine into the declaration list (same as a valid function), plus the placement diagnostic. Do not accept `static` as a language feature on programs. The diagnostic stays.

## Tests

Parser test: declarations length is 1, name `Foo`, and the invalid-static diagnostic is present.

## Verify

```text
cargo test -p fpas-parser
cargo fmt
```

## Done when

- Foo is in the AST.
- The placement error still fires.
- Docs unchanged.

## Progress

- Base commit: 74b16b7b
- Current step: verify recovered top-level static routine remains in the AST
- Files changed: `crates/fpas-parser/src/parser/decl/mod.rs`,
  `crates/fpas-parser/src/tests/errors/recovery.rs`
- Verification: not run
- Blockers: none
