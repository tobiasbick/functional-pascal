# AP37.2: Receiver mutation

Package: [AP37](README.md). Status: blocked by AP37.1.

## Scope and affected areas

Implement the chosen receiver model without weakening storage/lifetime checks.
Inspect `crates/fpas-sema/src/check/expr/calls/methods/mod.rs` and `lookup.rs`,
`check/decl/types/records/signatures.rs`, native catalogs, shared var-argument
checking, compiler reference lowering, and formatter/LSP handling. Enumerate
concrete files before editing.

## Migration and documentation

Migrate affected native and user calls with resolved writable roots and
single evaluation of indices and arguments. Update
[var parameters](../../../pascal/language/functions/var-parameters.md),
[native operations](../../../pascal/language/functions/fluent-calls.md),
record methods, collection APIs, grammar, compact reference, and editor tools.
Do not remove copy-returning operations merely because mutation also exists.

## Verification

Test writable and immutable roots, nested record/array storage, aliasing,
effectful indices, callback/bound-method lifetimes, and forbidden task
crossings. Verify that mutation affects the intended caller storage while
copy-producing APIs preserve their inputs. Run required implementation checks.
