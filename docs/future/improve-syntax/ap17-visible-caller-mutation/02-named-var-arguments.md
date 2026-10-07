# AP17.2: Named var arguments

Package: [AP17: Visible caller mutation](README.md)

## Scope

Allow the named form `Increase(Value := var Counter)`.

## Prerequisites

- AP17.1 (`var` parameters).
- AP09.1 (named arguments).

## Implementation

- Parser: `Name := var Designator` in named argument lists.
- Sema: apply the AP17.1 argument, aliasing, and evaluation rules to named
  arguments in written order.
- Formatter and signature help.

## Affected areas

- Parser call arguments, sema named-argument mapping, `fpas-fmt`.

## Migration

None.

## Documentation

- `var-parameters.md`, `parameters.md`.
- `docs/specs/grammar.ebnf` (`named_arg`).

## Verification

- Named `var` arguments in any order, aliasing rejection across named
  arguments, missing marker in named form, evaluation-order traces.
- Writable storage, exact types, forwarding, generic routines, methods,
  imported routines with compiled-unit reuse, and writes retained on `try` exit.
- Rejections: marker on a read-only parameter, invalid writable storage,
  invalid name mappings, mixed calls, function values, and `go` calls.
- Formatter round trip and comments, signature help, and separate rename of
  parameter labels and referenced variables; regular CLI and FPAS test runs.
