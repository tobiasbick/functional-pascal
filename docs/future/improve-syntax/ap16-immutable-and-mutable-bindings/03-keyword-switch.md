# AP16.3: Keyword switch to const and var

Package: [AP16: Immutable and mutable bindings](README.md)

Status: complete.

## Result

`const` is immutable and `var` is reassignable at program, unit and local scope.
Value parameters and loop variables are read-only; AP17 reference parameters
use `var`. Captured local `var` bindings share cells. `mutable` is an ordinary
identifier; retired declaration prefixes receive contextual correction hints.

Local parameter changes use fresh local copies, preserving writes,
captures, shadowing and member names without introducing caller mutation.
Parser, sema, persisted parameter types, source consumers, generated APIs,
formatter, language service, snippets and highlighting use the current forms.
Writable exported variables and read-only constants retain their interface and
linking behavior.

## Regression coverage

Tests cover keyword use, reassignment, constants, parameters, captures, field/
element writes, imported variables, diagnostics and formatting.
The [formatter handbook tests](../../../../crates/fpas-cli/src/main_tests/examples/formatter_handbook.rs)
extract the complete control-flow program directly from Markdown. CLI checks
and execution validate the mutable loop binding and expected output, and
formatter output must match the documented block. Restoring the original
immutable binding reproduces FP3005 without executing the invalid program.
See [variables](../../../pascal/language/basics/variables.md) and
[parameters](../../../pascal/language/functions/parameters.md).
