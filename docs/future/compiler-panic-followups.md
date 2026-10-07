# Compiler panics and language-limit follow-ups

This document is the intake list for compiler panics and language limits discovered during normal
FPAS work. Resolved behavior belongs in the implementation, regression suite, and current
documentation; broader language proposals belong in their own future notes.

## Entry rule

When development finds a compiler panic or a language limitation that requires a workaround, add an
entry here in the same change. Include the source shape, observed failure or restriction, temporary
workaround, and a concrete later resolution with regression coverage. Do not silently extend the
language or hide the limitation inside a library implementation.

## Open entries

### Debugger evaluation accepts positional call arguments only

- **Source shape:** a debugger `evaluate` expression with named arguments, for
  example `Sub(Left := 1, Right := 2)` or `Shape.Rect(Width := 1.0, Height := 2.0)`.
- **Restriction:** `crates/fpas-debug/src/evaluation/validate.rs` rejects
  `Expr::NamedArgument`; the debugger resolves calls without semantic analysis
  and therefore has no parameter-name mapping.
- **Workaround:** pass the arguments by position in declaration order.
- **Resolution:** map names to the callee's parameter or variant-field names
  from the debug metadata, evaluate in written order, and add debugger
  evaluation tests for routines, methods, and variant constructors.
