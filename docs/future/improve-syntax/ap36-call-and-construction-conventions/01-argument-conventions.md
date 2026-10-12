# AP36.1: Argument conventions

Package: [AP36](README.md). Status: Decision pending; decision work package.

## Scope and deliverable

Compare positional-only routine calls with consistent named-call support.
Include free/imported routines, methods, bound methods, callable fields and
parameters, anonymous functions, native operations, polymorphic/variadic
builtins, and `go`. Decide whether callable labels form part of compatibility
and whether unused names should remain mandatory in callable type syntax.
Evaluate named-only user aggregate construction and named versus positional
patterns separately, including nested and built-in variants.

## Prerequisites and affected areas

AP09/AP10/AP20 are the baseline. Inspect
`crates/fpas-sema/src/check/calls/named.rs`, callable type representation,
unit interfaces, and argument parsing. Planning files only are changed.
Do not reopen AP08's parameter separator decision or AP12's direct returned
callable syntax as incidental cleanup.

## Documentation and verification

Record the user's choice and affected earlier decisions. The example matrix
must cover same-typed parameter roles, callback substitution, reordered
arguments with effects, var markers, and constructor/pattern symmetry.
Current handbook and grammar remain unchanged until AP36.2.
