# AP33.3: Number text

Package: [AP33](README.md). Status: blocked by AP33.1.

## Scope and dependencies

Implement AP33.1's text grammar, range, and formatting decisions for current
numeric types. Do not rename/remove parsing APIs here; AP43 owns that choice.
AP33.2 is required only if the selected text contract depends on its new
numeric representation or behavior; record that dependency before starting.

## Affected areas

Inspect `crates/fpas-std/src/numeric_text.rs`, `conv.rs`, `parse.rs`, lexer
numeric parsing, and native string numeric predicates. Reuse parsing rules
where the agreed contracts match; document intentional differences.

## Migration and documentation

Update [Std.Conv](../../../pascal/std/text/conv.md),
[Std.Parse](../../../pascal/std/text/parse.md), numeric string operations,
literal documentation, and examples. Migrate consumers affected by changed
formatting or accepted input without introducing a legacy parsing mode.

## Verification

Check finite formatting/parsing round-trips including integral real values,
small/large exponents, signed zero if preserved, and boundary integers.
Test whitespace, underscores, hex input, overflow, nonzero underflow,
nonfinite text, and agreement of numeric predicates with their documented
parser. Verify both Result-returning and aborting wrappers while both exist.
Run relevant Std, lexer, FPAS, and repository checks.
