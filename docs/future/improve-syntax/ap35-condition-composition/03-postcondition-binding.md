# AP35.3: Postcondition binding

Package: [AP35](README.md). Status: complete.

## Implemented behavior

`ensures (Name): Expression;` is the only return-binder spelling. The parser
recognizes a binder only from the complete `(identifier):` prefix. Otherwise
it parses an ordinary expression: `ensures (Flag);`, compound grouping, and
grouped field/index/query operations retain their normal meaning.

Adjacent legacy binder conditions are rejected with a concrete colon hint,
including conditions beginning with Option/Result constructors or native array
factories. A colon-free expression that is valid ordinary syntax introduces
no result name. Procedures reject result binders.

AP23/Q16's binder punctuation is replaced. The complete declared return type,
immutability, clause-only scope, return timing, propagated Option/Result exits,
and contract expression allowlist are preserved.

## Implementation owners

- `fpas-parser/src/parser/decl/contracts.rs` and `ast/contracts.rs`: complete
  prefix recognition, clause text/spans, and bounded legacy recovery.
- `fpas-fmt/src/emit/decl/contracts.rs`: mandatory binder colon and stable comments.
- Existing Sema contract checks, compiler exit lowering, unit interfaces, and
  language-service clause symbols retain type, timing, and lexical identity.

## Migration and documentation

All positive source and Rust fixtures, formatter examples, handbook/reference
clauses, authoring guidance, and AP23 records use the colon. Ordinary grouped
conditions and binder-free clauses remain expressions.

Current behavior: [contracts](../../../pascal/language/functions/contracts.md),
formal grammar, compact reference, formatter style, and authoring skill.

## Regression coverage

Parser clause tests cover grouped conditions, named binders, source retention,
and legacy migration hints. Formatter tests cover punctuation and comment
round trips. Sema covers result types, procedure rejection, clause scope, and
recursive operand restrictions. Compiler postcondition tests cover grouped
Boolean parameters, early/final and propagated returns, body shadowing,
clause ordering, and writes. Editor tests preserve clause-local identities
and outer grouped-name navigation; CLI/interface/FPAS tests cover persistence
and complete return values. Independent review confirms the binder boundary.
