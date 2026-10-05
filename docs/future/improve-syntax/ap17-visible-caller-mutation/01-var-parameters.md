# AP17.1: var parameters and arguments

Package: [AP17: Visible caller mutation](README.md)

## Scope

Add `var` parameters and positional `var` arguments with argument validity,
aliasing rejection, capture and `go` restrictions, and forwarding.

## Prerequisites

- AP16.3 (keyword switch; `var` bindings are reassignable).
- The open decision on function types in the [package README](README.md).
- AP13: recheck at start whether a block-syntax work package is required.

## Implementation

- Parser and AST: `var` before a parameter and before a call argument.
- Sema: a `var` parameter requires a `var` argument and vice versa; the
  argument is a `var` binding or a field or element of one; two `var`
  arguments of one call must not share a root variable; closures cannot
  capture `var` parameters; `go` calls cannot take `var` arguments; a `var`
  parameter can be forwarded as `var Value`.
- Specify and implement evaluation: the root and indices of each `var`
  argument are evaluated once, left to right with the other arguments.
  Specify the visible effect on the caller's variable when the routine fails
  (panic or `try` exit) and record it in the package README before
  implementing it; the reference branch retained writes made before the
  failure.
- Carry the parameter mode through unit interfaces, IR, bytecode, and VM;
  incompatible compiled units are rebuilt.
- Function types follow the open decision in the package README.
- Diagnostics: a missing `var` at the call site shows the corrected call; a
  `var` argument for a read-only parameter is rejected.
- Debugger writes follow the same rules.

## Affected areas

- Parser routine headings and call arguments; `crates/fpas-sema/src/check/`
  (calls, a focused references module, closures); `fpas-unit` interfaces;
  `fpas-ir`; `fpas-bytecode`; `fpas-vm`; `fpas-compiler/src/lowering/calls.rs`
  and aggregate path lowering; `fpas-fmt`; language-service signature help.

## Migration

None; no `var` parameters exist yet.

## Documentation

- New `docs/pascal/language/functions/var-parameters.md`; `parameters.md`,
  `function-types.md`, `closures.md`, concurrency `go.md`,
  `docs/specs/grammar.ebnf`.

## Verification

- Valid field and element arguments, forwarding, imported routines across
  compiled units, and function values as decided.
- Rejections: missing marker, marker on read-only parameter, `const` and
  temporary arguments, shared roots including `A[I]`/`A[J]`, captured `var`
  parameter, `go` with `var` argument.
- Evaluation order and partial effects on failure.
