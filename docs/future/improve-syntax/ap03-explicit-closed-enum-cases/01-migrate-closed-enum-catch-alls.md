# AP03.1: Migrate closed-enum catch-alls

Package: [AP03: Explicit closed-enum cases](README.md)

Status: complete.

## Result

Repository cases over user enums, `Option`, and `Result` use explicit variant
arms. Code handling one variant uses `is` where a full case is unnecessary.
Scalar cases over `integer`, `string`, and `boolean` retain their catch-alls.

Remaining variants are listed in declaration order. Guarded arms and partial
payload patterns retain an explicit fallback for the same variant when needed.
Payload wildcards ignore fields, while
no-action variants share a `null;` arm. Scrutinee evaluation, branch order,
returns, and local scopes are preserved.

## Ownership

- Source consumers: `lib/`, `apps/`, `examples/`, and `tests/`.
- Rust fixtures: compiler, semantic analysis, parser, formatter, and CLI tests.
- Handbook examples: [exhaustiveness](../../../pascal/language/pattern-matching/exhaustiveness.md),
  [scalar labels](../../../pascal/language/pattern-matching/scalar-labels.md),
  [`Std.Toml`](../../../pascal/std/text/toml.md), and
  [controlled text areas](../../../pascal/std/tui/text-area.md).

Positive examples and fixtures cover every closed-enum variant explicitly;
negative diagnostic fixtures intentionally exercise rejected catch-alls.
Compiler enforcement is owned by
[AP03.2](02-reject-closed-enum-catch-alls.md).

## Regression coverage

Existing Rust and FPAS suites cover nested patterns, constant comparisons,
invalid variant qualifiers, named-field diagnostics, guard order, single
scrutinee evaluation, local shadowing and capture, TUI routing, and unchanged
app results. Case-specific fixtures retain cases with explicit fallback arms.

Verification includes Rust and FPAS formatting, workspace build/tests, the
FPAS suite, affected app/example checks, and compilation of the changed
handbook examples.
