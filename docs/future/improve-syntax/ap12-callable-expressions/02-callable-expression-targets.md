# AP12.2: Callable expression targets

Package: [AP12: Callable expressions](README.md)

Status: deferred, inactive implementation sketch. Retained for possible later
reassessment; none of the additional syntax below is approved for implementation.

## Scope

The retained proposal would allow any correctly typed function expression as
a call target. Its scope must be reassessed if reopened; AP12.1 currently
records deferral, not acceptance.

## Prerequisites

- A new explicit user decision to reopen AP12, accepting specific call targets
  and evaluation order. The completed AP12.1 deferral does not satisfy this gate.
- AP06.3 (fixed dot-call resolution).

## Retained implementation sketch

- Parser: a call suffix after any primary or postfix expression.
- Sema: one value-call check for all callable targets; reject non-callable
  targets with a diagnostic naming the target's type.
- Compiler: evaluate the target once, then the arguments left to right;
  reuse the existing value-call instruction.
- Formatter and signature help for computed targets.

## Affected areas

- `crates/fpas-parser/src/parser/expr/postfix.rs`.
- `crates/fpas-sema/src/check/expr/postfix.rs`, call checking.
- `crates/fpas-compiler/src/lowering/calls.rs`.
- `fpas-fmt`, `crates/fpas-language-service/src/intellisense/signature_help.rs`.

## Migration

None while deferred. Any future implementation would be additive and require
an audit of the then-current supported targets.

## Documentation

- `docs/pascal/language/functions/first-class.md`, `function-types.md`,
  `postfix-chaining.md`, `docs/specs/grammar.ebnf`.

## Verification

- Returned functions, selected function values, indexed values, function
  fields, parenthesized anonymous functions; evaluation-order traces; `try`
  in arguments; non-callable targets; procedure values in statement position.
