# AP33.3: Number text

Package: [AP33](README.md). Status: complete.

## Scope and dependencies

Source literals and numeric text share decimal/`$` hexadecimal integer bodies
and real bodies with a required fractional part, optional exponent and single
underscore separators. Input adds an optional leading sign and surrounding
whitespace. Integer input covers the full signed range. Nonfinite real input,
overflow and nonzero input rounded to zero are invalid; representable subnormals
remain valid. Real formatting parses back to identical binary64 bits, including
integral real values and signed zero. Existing public API names remain; AP43
owns their Result/error-handling convention.

## Implementation ownership

[`Shared numeric text`](../../../../crates/fpas-numeric/src/text.rs) owns body
grammar, signed input, bounds and finite real formatting. The lexer uses its
unsigned body parsers. [`Std.Conv`](../../../../crates/fpas-std/src/conv/mod.rs),
[`Std.Parse`](../../../../crates/fpas-std/src/parse.rs) and
[`string numeric predicates`](../../../../crates/fpas-std/src/str/mod.rs) use
the same input parsers. HexToInt additionally accepts bare hex and 0x/0X prefixes
through its explicit hex adapter. Boolean text has its separate owner.

[`TOML conversion`](../../../../crates/fpas-std/src/toml/mod.rs) rejects nonfinite
values recursively and returns Result errors. The TOML parser otherwise retains
its own documented syntax. AP33.2 supplies matching arithmetic bounds and
literal handling without changing numeric representation.

## Migration and documentation

[Std.Conv](../../../pascal/std/text/conv.md),
[Std.Parse](../../../pascal/std/text/parse.md),
[numeric string operations](../../../pascal/language/types/string/format-chars.md),
[literals](../../../pascal/language/basics/number-literals.md),
[TOML](../../../pascal/std/text/toml.md), and generated API comments describe
the current forms and error boundaries. TOML fixtures use finite signed zero
and retain their structural assertions.

## Regression coverage

[`Shared text tests`](../../../../crates/fpas-numeric/src/tests/text.rs) own grammar
and exact binary64 round-trips, including signed zero, subnormal and maximum
finite values. Compiler numeric contracts exercise public parsing/formatting.
[`FPAS conversion tests`](../../../../tests/stdlib/conv/numeric_text_contracts_test.fpas)
verify conversion, Result and predicate routing through the installed unit graph.
Existing direct-conversion tests retain aborting-wrapper error checks.
[`TOML tests`](../../../../crates/fpas-std/src/toml/tests.rs) reject top-level and
nested nonfinite values. Changed documentation examples are valid FPAS sources.

## Open follow-ups

None within this work package's scope.
