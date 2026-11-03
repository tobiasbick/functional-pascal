# Scope, recursion, and effects

Free declarations are private to their unit unless `public`. Record members are
public within their record's accessibility unless explicitly private. Private
members are accessible only from inside the record. A static method has no self
and does not capture an instance. Static factories may construct private record
state. Signatures are visible throughout their lexical declaration scope,
permitting mutual recursion. Binding values
initialize in source order; reads before definite assignment are errors.

Functions may perform I/O, mutate objects, and throw exceptions. No purity effect
system is implied by `function`. Contract conditions use a restricted read-only
expression subset; they do not declare the whole function pure. Use contracts
for executable expectations, ordinary CLR exceptions for failures, and explicit
domain outcomes where appropriate.
Tail-call optimization may be performed, but stack safety is not guaranteed by
the language; use loops for unbounded iteration.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
