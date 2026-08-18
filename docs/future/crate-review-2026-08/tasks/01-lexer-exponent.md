# Task 01 — Reject leading `_` in numeric exponents

Status: open
Severity: P1
Difficulty: easy
Language gate: no
Depends on: none

## Goal

`1.0e_3` and `1.0e+_3` must be lexer errors. `1.0e3` and `1.0e-3` stay valid. `1_000` stays valid.

## Spec

[`docs/pascal/language/basics/number-literals.md`](../../../pascal/language/basics/number-literals.md) and `decimal_digits` in `specs/grammar.ebnf`: a digit run is `digit { ['_'] digit }`. An exponent must start with a digit, not `_`.

## Bug

`crates/fpas-lexer/src/lexer/numbers.rs` — `maybe_scan_exponent` calls `consume_decimal_digits` with the cursor on the first exponent character. `consume_digits_with` skips `_` when the next byte is a digit even if no digit has been read yet, so `1.0e_3` becomes `Real(1000.0)` with an empty error list.

## Fix

In `consume_digits_with` **or** in `maybe_scan_exponent` only: do not treat `_` as a separator until at least one digit of that run has been consumed. Keep the existing diagnostic for `1__2`. Do not change integer `_` separators.

Do not change the grammar file unless you discover it already forbids this (it should).

## Tests

Add to `crates/fpas-lexer/src/tests/errors/numeric.rs` next to `invalid_numeric_exponent_reports_explicit_help`:

- `1.0e_3` → at least one error; message mentions exponent or digit separator; must **not** produce `Token::Real(1000.0)` as a successful literal with zero errors.
- `1.0e+_3` → same.
- Keep `1.0e3` / `1.0e-3` succeeding (existing tests; do not break them).

## Verify

```text
cargo test -p fpas-lexer
cargo fmt
```

## Done when

- `1.0e_3` is a diagnostic, not the value 1000.
- Existing number tests still pass.
- Docs unchanged.
