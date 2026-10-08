# AP04.2: Require consumed function results

Package: [AP04: Discarded function values](README.md)

Status: complete.

## Result

Unused function results in statement position are rejected with FP3022,
including direct, member, intrinsic, callable-value and final postfix calls.
A result is consumed by binding, further computation, error handling, or a valid
explicit `discard`. Procedures and `go` statements remain standalone forms.

Hints use the same task-freedom proof as [AP04.1](01-discard-statement.md).
Task-containing values and unknown captures receive consumption guidance
without a suggestion to discard them. Repository callers consume or explicitly
discard results.

## Regression coverage

Sema, CLI and FPAS regressions cover ordinary, Option and Result values,
postfix calls, task-containing results, procedures and diagnostic hints.
See [postfix chaining](../../../pascal/language/functions/postfix-chaining.md).
