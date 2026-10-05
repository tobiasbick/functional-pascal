# AP12.2: Callable expression targets

Package: [AP12: Callable expressions](README.md)

## Scope

Allow any correctly typed function expression as a call target, as decided in
AP12.1.

## Prerequisites

- AP12.1 (accepted decision).
- AP06.3 (fixed dot-call resolution).

## Implementation

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
- `fpas-fmt`, `crates/fpas-language-service/src/intellisense/signature_help/`.

## Migration

None; the change is additive.

## Documentation

- `docs/pascal/language/functions/first-class.md`, `function-types.md`,
  `postfix-chaining.md`, `docs/specs/grammar.ebnf`.

## Verification

- Returned functions, selected function values, indexed values, function
  fields, parenthesized anonymous functions; evaluation-order traces; `try`
  in arguments; non-callable targets; procedure values in statement position.
