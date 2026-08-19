# Task 08 — Reject empty `const` / `var` / `type` / `enum` / `case` bodies

Status: complete
Severity: P2
Difficulty: easy
Language gate: no
Depends on: none

## Goal

A `const` / `var` / `mutable var` / `type` / `enum` / `case` section with **zero** members is a parse error. EBNF is one-or-more.

## Spec

[`docs/specs/grammar.ebnf`](../../../specs/grammar.ebnf): `const_def { const_def }`,
`enum_member { enum_member }`, `case_arm { ';' case_arm }`.

## Bug

`crates/fpas-parser/src/parser/decl/` loops while the next token looks like a member, so zero items succeed:

- `program T; const begin end.`
- same for `var`, `type`, `mutable var`
- `program T; const var X: integer := 1; begin end.` — stray `const` dropped
- `type E = enum end;`
- `case X of end`

Sites noted in review: `const_var.rs`, `type_defs.rs`, `branching.rs` (case).

## Fix

After entering the section, if the first member is missing, emit a diagnostic with a short example (`const X: integer := 1;`). Do not consume `begin` / `end` / `of` as a member. Keep recovering so the rest of the program still parses.

Do not invent a new error code if an existing `PARSE_*` code fits; otherwise add one in `fpas-diagnostics` and use it.

## Tests

`crates/fpas-parser/src/tests/` (errors module):

- Each empty construct produces at least one parser diagnostic.
- `const var X: integer := 1; begin end.` still parses the `var` **and** reports the empty/stray `const`.

## Verify

```text
cargo test -p fpas-parser
cargo fmt
```

## Done when

- Empty one-or-more constructs are errors.
- Valid programs with one member still parse.
- Docs unchanged (grammar already requires one-or-more).

## Progress

- Base commit: 74b16b7b
- Current step: verify empty-section diagnostics and recovery
- Files changed: parser declaration/case parsers and `src/tests/errors/recovery.rs`
- Verification: full workspace definition of done passed on 2026-08-19
- Blockers: none
