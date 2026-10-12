# AP38.1: Visibility decision

Package: [AP38](README.md). Status: Decision pending; decision work package.

## Scope and deliverable

Assess collecting all named signatures versus retaining source-order free
routines. Record the user's rule for free/member/nested routines and mutual
recursion. Specify value visibility independently: a forward call in an
initializer must not read a later uninitialized binding through a routine.
Cover record defaults, constants with runtime initializers, captured locals,
and public/imported signatures.

## Prerequisites and affected areas

AP11/AP24 are complete. Inspect declaration collection and checking under
`crates/fpas-sema/src/check/decl/`, routine body checking, and unit interfaces.
Change only planning files at this stage.

## Documentation and verification

Keep [declaration order](../../../pascal/language/types/declaration-order.md)
and [nested routines](../../../pascal/language/functions/nested.md) unchanged
until implementation. The recorded examples must predict forward helper
calls, mutually recursive routines, shadowing, and unsafe initialization.
