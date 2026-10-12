# AP36.2: Unified call rules

Package: [AP36](README.md). Status: blocked by AP36.1.

## Scope and affected areas

Implement the selected argument and construction rules in parser/sema,
callable signatures, serialized interfaces, call lowering, formatter, and
signature help. Start with `crates/fpas-sema/src/check/calls/named.rs`,
`crates/fpas-parser/src/ast/`, and `crates/fpas-sema/src/interface/`; list the
actual affected files after inspecting each selected representation change.

## Dependencies, migration, and documentation

AP36.1 must resolve all selected changes. AP34 is required only if the chosen
implementation relies on changed inference. Migrate every consumer using
resolved parameter/field roles, preserving written evaluation order with
temporaries where necessary. Update
[parameters](../../../pascal/language/functions/parameters.md), callable,
record, enum, pattern and native-operation pages, grammar, compact reference,
formatter examples, snippets, and exported API signatures.

## Verification

Check direct/callable/member/imported equivalents, callback compatibility,
native exceptions retained by decision, invalid labels, var arguments,
aggregate field roles, nested patterns, and side-effect ordering. Verify
sidecars and language-service signature help; run applicable repository checks.
