# AP06.2: Prepare type-operation and free-call migration

Package: [AP06: Fixed dot-call targets](README.md)

Status: complete.

## Result

Repository consumers use declared record members or native type operations for
dot calls. Own free routines and callable values use ordinary calls. Wrappers
provide callback references to native operations.

Factories, string `Slice`, named operations, imports, embedded/assembled Rust
programs, formatter fixtures, generators and documentation use current forms.
The five retired helper units have no public import, free-call or editor-stub
surface. Negative fixtures keep rejected forms only to check their diagnostics.

## Regression coverage

Compiler, sema, CLI, editor, FPAS and handbook checks cover consumer forms,
retired API rejection, chains, callable members and same-named free routines.
See [native resolution](03-fixed-dot-resolution.md) and
[catalog coverage](inventory.md).
