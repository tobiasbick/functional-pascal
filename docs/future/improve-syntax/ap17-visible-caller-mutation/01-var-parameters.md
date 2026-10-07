# AP17.1: var parameters and arguments

Package: [AP17: Visible caller mutation](README.md)

## Scope

Add `var` parameters and positional `var` arguments with argument validity,
aliasing rejection, capture and `go` restrictions, and forwarding.

## Prerequisites

- AP16.3 (keyword switch; `var` bindings are reassignable).
- AP13.3 and AP13.6 (routine and callable-expression closers; complete on
  the working branch).

## Implementation

- Parser and AST: `var` before a parameter in routine declarations and
  function types, and before a call argument.
- Sema: a `var` parameter requires a `var` argument and vice versa; the
  argument is a `var` binding or a field or element of one; two `var`
  arguments of one call must not share a root variable; closures cannot
  capture `var` parameters; `go` calls cannot take `var` arguments; a `var`
  parameter can be forwarded as `var Value`.
- Implement evaluation: the root and indices of each `var`
  argument are evaluated once, left to right with the other arguments.
  Assignments through a `var` parameter update the caller's variable as they
  execute. Preserve writes completed before a panic or `try` exit, including
  writes to field and element arguments; unwinding does not roll them back.
- Carry the parameter mode through unit interfaces, IR, bytecode, and VM;
  incompatible compiled units are rebuilt.
- Function types allow `var` parameters. Include parameter modes in
  function-type compatibility; reject substitution between read-only and
  `var` parameters. Calls through function values use positional arguments
  with explicit `var` markers and apply the same argument validity,
  aliasing, forwarding, capture, `go`, evaluation, and failure rules as
  calls to declared routines.
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
  compiled units, and function values with `var` parameters. Cover matching
  modes and reject function-type assignments with mismatched parameter
  modes in both directions, including across compiled-unit boundaries.
- Rejections: missing marker, marker on read-only parameter, `const` and
  temporary arguments, shared roots including `A[I]`/`A[J]`, captured `var`
  parameter, `go` with `var` argument.
- Evaluation order and single evaluation of argument roots and indices.
  Verify that writes before a panic or `try` exit remain visible for whole
  variables, fields, and elements, through declared routines and function
  values, while later writes do not execute.

## Result

Delivered with the agreed defaults for the open details:

- Unit and program variables, including public `var` variables of imported
  units, are valid `var` arguments.
- A closure may pass a `var` local it captures (`Increase(var Count)`).
- Named nested routines may use an enclosing `var` parameter when called
  directly; using such a routine as a value or with `go` is rejected (FP3030).
- Element arguments are array elements; dictionary entries and string
  characters are rejected (FP3028).
- Aliasing is checked among the arguments of one call; changes through other
  names, such as a unit variable written directly, are documented.
- Record methods accept `var` parameters; the receiver `Self` cannot be `var`.

Implementation: a `var` argument creates a runtime reference (root cell or
global plus field and element steps fixed at the call). Locals passed as `var`
are cell-backed. The IR type `Reference(T)` carries parameter modes through
function types, unit interfaces (`.fpascu` format 9), objects (version 9), and
bytecode (version 16).

Follow-up work recorded elsewhere:

- `Push`/`Pop` on a `var` parameter and receiver calls whose first parameter
  is `var` are rejected (FP3027); [AP17.3](03-caller-mutating-intrinsics.md)
  and [AP06.3](../ap06-dot-call-targets/03-fixed-dot-resolution.md) decide them.
- Debugger writes and calls with `var` parameters:
  [compiler and language-limit follow-ups](../../compiler-panic-followups.md).
