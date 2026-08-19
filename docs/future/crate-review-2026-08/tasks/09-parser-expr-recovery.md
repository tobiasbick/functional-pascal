# Task 09 — Do not consume sync tokens when an expression is missing

Status: in progress
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

`PARSE_EXPECTED_EXPRESSION` recovery must not `advance()` over tokens that close a parent construct: `end`, `then`, `do`, `until`, `else`, `)`, `]`, `;`, `.`

## Spec

Parser error recovery should keep following statements. Existing test `program T; begin X := end.` only checks that *some* expression error exists.

## Bug

`crates/fpas-parser/src/parser/expr/primary.rs` `parse_primary_atom`: on unexpected token it always `advance()`s, then returns `Expr::Error`. That steals `end` from `record … end`, `)` from calls, `]` from arrays, `then`/`do` from `if`/`while`.

Examples that desynchronize:

- `record X end; Y := 1 end.` — `Y := 1` swallowed as a field
- `Foo(1,)` / `[1,]` — closer treated as missing argument, then a second “missing `)`” error
- `if then` / `while do` — keyword eaten then demanded again

## Fix

If the current token is a statement/expression **sync token** (the list above, plus whatever `expect` already uses as closers), report the error and **do not** advance. Only advance on tokens that cannot help a parent (identifiers that are wrong, extra operators, etc.).

Look for an existing sync-set helper in the parser before adding a new list. Reuse it.

Do not try to “fix” the invalid program into a valid AST. Goal: later statements remain in the tree.

## Tests

Extend `crates/fpas-parser/src/tests/errors/diagnostics.rs` (the `X := end.` case) and add:

- After `program T; begin X := end; Y := 1 end.` the tree still contains the assignment to `Y` (or at least does not report a cascade that claims `.` is a field). Assert fewer cascading errors than today if you can do it stably; the must-have is: `end` / `)` / `]` are not consumed.
- `Foo(1,)` still reports missing expression **without** claiming `)` is missing, **or** reports both but the closer stays available (prefer one clear missing-arg error).

Dump the AST in the test if the crate already has a debug helper; otherwise assert diagnostic codes/counts and that a following `Y := 1` does not produce `PARSE_UNKNOWN_*` noise from stolen `end`.

## Verify

```text
cargo test -p fpas-parser
cargo fmt
```

## Done when

- Sync tokens stay in the stream.
- The existing `X := end.` test still sees `PARSE_EXPECTED_EXPRESSION`.
- Docs unchanged.

## Progress

- Base commit: 74b16b7b
- Current step: verify expression-boundary recovery regressions
- Files changed: `crates/fpas-parser/src/parser/core.rs`,
  `crates/fpas-parser/src/parser/expr/primary.rs`, parser error tests
- Verification: not run
- Blockers: none
