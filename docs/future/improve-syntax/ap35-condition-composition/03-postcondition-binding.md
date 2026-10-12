# AP35.3: Postcondition binding

Package: [AP35](README.md). Status: blocked by the binder decision in AP35.1.

## Scope and affected areas

Implement an unambiguous return binder while preserving postcondition timing
and per-clause scope. Inspect `crates/fpas-parser/src/parser/decl/contracts.rs`,
`crates/fpas-sema/src/check/contracts/`, formatter output, and editor grammar.
Do not broaden the contract expression language or resurrect AP25.

## Migration and documentation

Migrate every existing binder and formatter fixture in the same delivery.
Update [contracts](../../../pascal/language/functions/contracts.md), formal
grammar, compact reference, snippets, diagnostics, and relevant AP23 records.
Clearly distinguish selected syntax from the discarded candidate forms.

## Verification

Check single-name and compound parenthesized Boolean conditions, named result
access, multiple clauses, procedure rejection of a result binder, early/final
returns, propagated Option/Result exits, and postconditions after scope exit.
Retain the existing native-expression allowlist and negative call tests.
Run parser/formatter/sema/runtime and required repository checks.
