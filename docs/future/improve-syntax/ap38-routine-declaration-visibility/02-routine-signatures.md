# AP38.2: Routine signatures

Package: [AP38](README.md). Status: blocked by AP38.1.

## Scope and affected areas

Implement the selected signature visibility with separate initialization
checks. Inspect sema declaration collection, nested scope construction,
routine body checks, compiler symbol allocation, unit interfaces, and LSP
resolution/navigation. Reuse the existing member/type collection machinery
where appropriate rather than creating competing name-resolution passes.

## Migration and documentation

Preserve or explicitly migrate value initialization order. Do not reorganize
unrelated routines solely for style. Update
[declaration order](../../../pascal/language/types/declaration-order.md),
record defaults/methods, nested routines, units, and compact reference.
If the decision retains existing behavior, replace this implementation scope
with its bounded clarification work before marking anything complete.

## Verification

Test forward free/member calls, nested shadowing, mutual recursion, later
values, initializer calls, captures, duplicate names, and independent unit
compilation. Ensure editor references resolve to the same declaration as
the compiler. Run applicable workspace and FPAS checks.
