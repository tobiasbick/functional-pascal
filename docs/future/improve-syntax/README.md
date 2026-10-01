# Syntax improvement implementation plan

This is the steering document for planned Functional Pascal changes. It is the
starting point for changing the language specification, not the current
specification itself, and it does not authorize implementation of unresolved
language changes. Active work packages remain unchecked until their
implementation, migration, documentation, and verification are complete.

## Design boundary

Keep the language recognizably part of the Pascal family: case-insensitive
names, `:=`, `function`, `procedure`, `record`, `case`, `begin`, `end`, and
word-based blocks. Several packages borrow Ada conventions (named block
endings, `var` parameters marked at the call site, per-declaration keywords).
Where Pascal or Delphi already has a spelling for a concept, prefer it; use
Ada only where the Pascal family has no established form.

Keep `array of T` and the `of` type application (`Option of T`,
`Result of (T, E)`, `task of T`). User-defined generic types use the same `of`
form; do not introduce a second angle-bracket application syntax for types.
Prefer regular rules, visible argument roles, explicit mutation, and concrete
diagnostics.

Existing features may be simplified or removed, but migration must preserve
behavior unless a semantic change is explicitly agreed. Implementation
languages and target platforms are outside this plan's scope.

**Agreed direction** records a design direction from the source draft, not a
complete grammar or proof of implementation. **Proposal** requires an explicit
decision before language implementation. Open details in either category must
be resolved with the user before changing syntax or semantics.

Every code fragment in these files is draft syntax, not a compiled example or
a promise that the current compiler accepts it. Current behavior must be checked
against the checkout before implementing each package.

## Work-package checklist

Effort estimates are preliminary, based on the design rather than a code audit.
Dependencies identify rule contracts that must be settled; verified existing
support may satisfy them without rebuilding it.

| Done | Package | Effort | Depends on | Direction |
|------|---------|--------|------------|-----------|
| [ ] | [AP01: Pascal conventions](01-conventions-and-diagnostics.md#ap01-pascal-conventions) | Small | None | Agreed direction |
| [ ] | [AP02: Structured diagnostics](01-conventions-and-diagnostics.md#ap02-structured-diagnostics) | Small | None | Agreed direction (Q01) |
| [ ] | [AP03: Explicit closed-enum cases](01-conventions-and-diagnostics.md#ap03-explicit-closed-enum-cases) | Small | AP02, AP20 | Agreed direction |
| [ ] | [AP04: Discarded function values](01-conventions-and-diagnostics.md#ap04-discarded-function-values) | Small | AP02 | Agreed direction |
| [ ] | [AP05: Qualified imports](02-resolution-and-argument-lists.md#ap05-qualified-imports) | Medium | AP01, AP02 | Agreed direction (Q04) |
| [ ] | [AP06: Remove automatic receivers](02-resolution-and-argument-lists.md#ap06-remove-automatic-receivers) | Medium | AP05 | Agreed direction |
| [ ] | [AP07: Boolean rules](02-resolution-and-argument-lists.md#ap07-boolean-rules) | Medium | AP02 | Agreed direction |
| — | [AP08: Comma-separated parameter lists](02-resolution-and-argument-lists.md#ap08-comma-separated-parameter-lists) | — | — | Rejected; closed (Q06) |
| [ ] | [AP09: Named arguments](03-construction-and-blocks.md#ap09-named-arguments) | Medium | AP02 | Agreed direction |
| [ ] | [AP10: Typed record construction](03-construction-and-blocks.md#ap10-typed-record-construction) | Medium | AP09 | Agreed direction |
| [ ] | [AP11: Individual declarations](03-construction-and-blocks.md#ap11-individual-declarations) | Medium | AP01 | Agreed direction |
| [ ] | [AP12: Callable expressions](03-construction-and-blocks.md#ap12-callable-expressions) | Medium | AP06 | Proposal |
| [ ] | [AP13: Explicit block boundaries](03-construction-and-blocks.md#ap13-explicit-block-boundaries) | Large | AP01, AP02 | Agreed direction |
| [ ] | [AP14: Remove computed properties](04-members-and-mutation.md#ap14-remove-computed-properties) | Small | AP06 | Agreed direction |
| [ ] | [AP15: Remove event declarations](04-members-and-mutation.md#ap15-remove-event-declarations) | Small | AP20 | Agreed direction |
| [ ] | [AP16: Immutable and mutable bindings](04-members-and-mutation.md#ap16-immutable-and-mutable-bindings) | Large | AP11 | Agreed direction |
| [ ] | [AP17: Visible caller mutation](04-members-and-mutation.md#ap17-visible-caller-mutation) | Large | AP09, AP13, AP16 | Agreed direction |
| [ ] | [AP18: Subrange types](05-types-and-contracts.md#ap18-subrange-types) | Large | AP07, AP16 | Agreed direction |
| [ ] | [AP19: Distinct domain types](05-types-and-contracts.md#ap19-distinct-domain-types) | Large | AP05, AP16 | Agreed direction (Q12, Q13) |
| [ ] | [AP20: Nested patterns and explicit bindings](05-types-and-contracts.md#ap20-nested-patterns-and-explicit-bindings) | Large | AP13 | Agreed direction |
| [ ] | [AP21: Decision expressions](05-types-and-contracts.md#ap21-decision-expressions) | Large | AP03, AP07, AP13 | Agreed direction |
| [ ] | [AP22: Limited local inference](05-types-and-contracts.md#ap22-limited-local-inference) | Large | AP02, AP16 | Agreed direction (Q14) |
| [ ] | [AP23: Preconditions and postconditions](05-types-and-contracts.md#ap23-preconditions-and-postconditions) | Large | AP07, AP13, AP16 | Agreed direction |
| [ ] | [AP24: Generic data structures](05-types-and-contracts.md#ap24-generic-data-structures) | Very large | AP10, AP20 | Agreed direction |
| [ ] | [AP25: Conservative purity](06-purity-and-scopes.md#ap25-conservative-purity) | Very large | AP14, AP16, AP17 | Retained; low priority; reassess after AP23 (Q18) |
| [ ] | [AP26: Structured task scopes](06-purity-and-scopes.md#ap26-structured-task-scopes) | Large | AP13, AP17 | Agreed direction (Q19, Q20) |
| [ ] | [AP27: Typed placeholders](07-tooling-and-derivation.md#ap27-typed-placeholders) | Very large | AP02, AP22 | Retained as optional (Q21); AP22 implementation required |
| — | [AP28: Structured editing](07-tooling-and-derivation.md#ap28-structured-editing) | — | AP13, AP24 | Outside language plan; transfer to editor/LSP planning afterward (Q23) |
| — | [AP29: Bounded derivation](07-tooling-and-derivation.md#ap29-bounded-derivation) | — | — | Rejected; closed (Q22) |

Use each package's task list to track implementation. Change its central Done
cell to `[x]` only when every task and its acceptance criterion are satisfied.
This table is the only package-completion index; do not create a second backlog.
Rejected packages use `—` in the Done column and are closed without implementation.
Work deferred to another plan also uses `—`; its destination and transfer
conditions are recorded in the owning section.

## Recommended order

Package numbers group related work; they are not the execution order. Ordered
by expected LLM benefit relative to cost:

1. AP01 and AP02: establish conventions and improve diagnostics.
2. AP07 and AP04: small changes that remove frequent silent mistakes.
3. AP16 together with AP17: align binding and parameter keywords with Pascal
   expectations in one migration.
4. AP13: the largest migration; settle its open details first.
5. AP09, AP10, and AP14.
6. AP20, then AP03 and AP15; AP21 and AP24.
7. AP18, AP19, and AP23.
8. AP26 and AP25.
9. AP05, AP06, and AP12 once their open decisions are settled.

AP27 is optional and follows only after its dependencies. AP28 is retained as
editor/LSP work outside this language plan, for transfer after AP13 and AP24.
AP29 is rejected.

## Execution and decision gates

- [ ] Identify the smallest ready package in the recommended order.
- [ ] Resolve that package's open language decisions with the user.
- [ ] Inspect its owning modules, file sizes, existing features, and tests.
- [ ] List actual files to create, modify, split, move, or remove before edits.
- [ ] Implement one package or explicitly bounded substep; migrate affected callers.
- [ ] Add positive, negative, boundary, and relevant end-to-end coverage.
- [ ] Update current documentation only for implemented behavior.
- [ ] Run the repository's applicable formatting, build, and test checks; report limits.
- [ ] Apply the package's acceptance criterion and the
  [shared regression checks](shared-constraints.md).
- [ ] Record current status, decisions, affected files, evidence, checks/results,
  remaining blockers, and the next step in the owning package. Do not add dates,
  personal or machine metadata, or a chronological change log.

These workflow boxes apply to the selected package, not to completion of the
entire roadmap. Distinguish **direction agreed**, **specified**, **implemented**,
and **verified**; documentation alone does not complete implementation.

The retained packages in AP01–AP12 contain independently useful improvements.
AP16–AP17 require binding and parameter migration. AP27 is optional.

AP22 is approved only for literals, typed construction, and explicit conversions
(Q14); ordinary calls still require annotations. AP27 retains its implementation
dependency on AP22 and is retained as optional (Q21). Do not broaden
inference indirectly through another package.

## Excluded and deferred

**Excluded:** an effect system is not part of this plan. This covers deferred
workflow values, declared service requirements and provision, effect routines
with `perform`, and effect-based retry/timeout composition. Errors stay with
`Result`, `Option`, and `try`; dependencies are passed as ordinary parameters,
for example records of function values; concurrency stays with `go` and
`task of T`, structured by AP26.

**Deferred:** general laziness, automatic partial application, custom
operators, a broad typeclass model, another lambda notation, open macros, and
library-specific DSLs are not part of the initial plan. Keep existing test
routines and sidecars; native test blocks require separate evidence and
agreement. Do not add a feature solely because an inspiration language has it.

## Document map

The numbered documents contain the work packages; [shared-constraints.md](shared-constraints.md)
contains cross-package rules, regression checks, and design inspiration.
All 23 questionnaire decisions are recorded in the owning packages as Q01–Q23.
These decisions do not mark implementation complete; remaining implementation
details and explicit decision gates stay in the owning packages.
Implemented behavior remains in the [current handbook](../../pascal/README.md).
