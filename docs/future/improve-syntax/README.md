# Syntax improvement implementation plan

This is the steering document for planned Functional Pascal changes. It is the
starting point for changing the language specification, not the current
specification itself, and it does not authorize implementation of unresolved
language changes.

Each package (`APnn`) has its own directory with a `README.md` entry point and
one file per work package (`APnn.m`). This document holds the overview and the
package status; [development-process.md](development-process.md) defines how a
work package is branched, merged, and marked done;
[shared-constraints.md](shared-constraints.md) holds cross-package rules and
regression checks.

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
decision before language implementation. Open details in either category are
listed under **Open decisions** in the owning package and must be resolved with
the user before changing syntax or semantics.

Every code fragment in these files is draft syntax, not a compiled example or
a promise that the current compiler accepts it. Current behavior must be checked
against the checkout before implementing each work package.

## Package status

AP01 and AP02 are delivered locally on `codex/syntax-changes-2`; their package
READMEs record the results. Their merge completion checkboxes remain open under
the [status-tracking rule](development-process.md#status-tracking).

Effort estimates are preliminary, based on the design rather than a code audit.
Dependencies name the packages a package needs; the real prerequisites per work
package are listed in each work package file.

| Done | Package | Effort | Depends on | Direction |
|------|---------|--------|------------|-----------|
| [ ] | [AP01: Pascal conventions](ap01-pascal-conventions/README.md) | Small | None | Agreed direction |
| [ ] | [AP02: Structured diagnostics](ap02-structured-diagnostics/README.md) | Small | None | Agreed direction (Q01) |
| [ ] | [AP03: Explicit closed-enum cases](ap03-explicit-closed-enum-cases/README.md) | Small | AP02, AP20 | Agreed direction |
| [ ] | [AP04: Discarded function values](ap04-discarded-function-values/README.md) | Small | AP02 | Agreed direction |
| [ ] | [AP05: Qualified imports](ap05-qualified-imports/README.md) | Medium | AP01, AP02 | Agreed direction (Q04) |
| [ ] | [AP06: Fixed dot-call targets](ap06-dot-call-targets/README.md) | Medium | AP05 | Agreed direction (Q05, revised) |
| [ ] | [AP07: Boolean rules](ap07-boolean-rules/README.md) | Medium | AP02 | Agreed direction (Q02, Q03) |
| — | [AP08: Comma-separated parameter lists](ap08-comma-separated-parameter-lists/README.md) | — | — | Rejected; closed (Q06) |
| [ ] | [AP09: Named arguments](ap09-named-arguments/README.md) | Medium | AP02 | Agreed direction |
| [ ] | [AP10: Typed record construction](ap10-typed-record-construction/README.md) | Medium | AP09 | Agreed direction |
| [ ] | [AP11: Individual declarations](ap11-individual-declarations/README.md) | Medium | AP01 | Agreed direction (Q07) |
| [ ] | [AP12: Callable expressions](ap12-callable-expressions/README.md) | Medium | AP06 | Proposal |
| [ ] | [AP13: Explicit block boundaries](ap13-explicit-block-boundaries/README.md) | Large | AP01, AP02 | Agreed direction (Q08, Q09) |
| [ ] | [AP14: Remove computed properties](ap14-remove-computed-properties/README.md) | Small | AP06 | Agreed direction |
| [ ] | [AP15: Remove event declarations](ap15-remove-event-declarations/README.md) | Small | AP20 | Agreed direction |
| [ ] | [AP16: Immutable and mutable bindings](ap16-immutable-and-mutable-bindings/README.md) | Large | AP11 | Agreed direction |
| [ ] | [AP17: Visible caller mutation](ap17-visible-caller-mutation/README.md) | Large | AP09, AP13, AP16 | Agreed direction |
| [ ] | [AP18: Subrange types](ap18-subrange-types/README.md) | Large | AP07, AP16 | Agreed direction (Q10, Q11) |
| [ ] | [AP19: Distinct domain types](ap19-distinct-domain-types/README.md) | Large | AP05, AP16 | Agreed direction (Q12, Q13) |
| [ ] | [AP20: Nested patterns and explicit bindings](ap20-nested-patterns-and-explicit-bindings/README.md) | Large | AP13 | Agreed direction |
| [ ] | [AP21: Decision expressions](ap21-decision-expressions/README.md) | Large | AP03, AP07, AP13 | Agreed direction |
| [ ] | [AP22: Limited local inference](ap22-limited-local-inference/README.md) | Large | AP02, AP10, AP16 | Agreed direction (Q14) |
| [ ] | [AP23: Preconditions and postconditions](ap23-preconditions-and-postconditions/README.md) | Large | AP07, AP13, AP16 | Agreed direction (Q15–Q17) |
| [ ] | [AP24: Generic data structures](ap24-generic-data-structures/README.md) | Very large | AP10, AP20 | Agreed direction |
| [ ] | [AP25: Conservative purity](ap25-conservative-purity/README.md) | Very large | AP14, AP16, AP17 | Retained; low priority; reassess after AP23 (Q18) |
| [ ] | [AP26: Structured task scopes](ap26-structured-task-scopes/README.md) | Large | AP13, AP17 | Agreed direction (Q19, Q20) |
| [ ] | [AP27: Typed placeholders](ap27-typed-placeholders/README.md) | Very large | AP02, AP22 | Retained as optional (Q21); AP22 implementation required |
| — | [AP28: Structured editing](ap28-structured-editing/README.md) | — | AP13, AP24 | Outside language plan; transfer to editor/LSP planning afterward (Q23) |
| — | [AP29: Bounded derivation](ap29-bounded-derivation/README.md) | — | — | Rejected; closed (Q22) |

A package's Done cell becomes `[x]` only when every work package in its own
checklist is done. This table is the only package-completion index; do not
create a second backlog. Rejected packages use `—` and are closed without
implementation. Work deferred to another plan also uses `—`; its destination
and transfer conditions are recorded in the owning package README.

## Work package index

| Package | Work packages |
|---------|---------------|
| AP01 | AP01.1 reference examples; AP01.2 plan spelling review |
| AP02 | AP02.1 audit and code scheme; AP02.2 shared record; AP02.3 project/build diagnostics; AP02.4 JSON output; AP02.5 reference; AP02.6 parameter declaration diagnostics |
| AP03 | AP03.1 migrate catch-alls; AP03.2 reject catch-alls |
| AP04 | AP04.1 discard statement; AP04.2 require consumed results |
| AP05 | AP05.1 import aliases; AP05.2 alias-aware tooling |
| AP06 | AP06.1 catalog and rules decision; AP06.2 migrate non-catalog calls; AP06.3 fixed dot resolution |
| AP07 | AP07.1 short-circuit evaluation; AP07.2 `Std.Bits`; AP07.3 logical precedence; AP07.4 boolean-only operators |
| AP09 | AP09.1 routine arguments; AP09.2 variant constructors |
| AP10 | AP10.1 typed construction; AP10.2 migrate literals; AP10.3 remove literals |
| AP11 | AP11.1 order-independent types; AP11.2 one keyword per declaration |
| AP12 | AP12.1 decision; AP12.2 callable targets |
| AP13 | AP13.1 reserve keywords; AP13.2 terminators; AP13.3 declaration closers; AP13.4 conditionals and loops; AP13.5 case arms; AP13.6 expression closers |
| AP14 | AP14.1 migrate properties; AP14.2 remove declarations |
| AP15 | AP15.1 migrate events; AP15.2 remove declarations |
| AP16 | AP16.1 computed const; AP16.2 migrate immutable var; AP16.3 keyword switch |
| AP17 | AP17.1 var parameters; AP17.2 named var arguments; AP17.3 mutating intrinsics |
| AP18 | AP18.1 declarations and conversions; AP18.2 membership |
| AP19 | AP19.1 declarations and conversions; AP19.2 comparisons |
| AP20 | AP20.1 explicit bindings; AP20.2 nested patterns; AP20.3 `is` test |
| AP21 | AP21.1 `if` expressions; AP21.2 `case` expressions |
| AP22 | AP22.1 literals and conversions; AP22.2 typed construction |
| AP23 | AP23.1 preconditions; AP23.2 postconditions |
| AP24 | AP24.1 parenthesized type arguments; AP24.2 generic records; AP24.3 generic enums |
| AP25 | AP25.1 priority reassessment; AP25.2 pure function checker; AP25.3 std purity metadata |
| AP26 | AP26.1 scope blocks; AP26.2 failure propagation; AP26.3 handle escape restrictions |
| AP27 | AP27.1 spelling decision; AP27.2 placeholder analysis |

AP08, AP28, and AP29 have no work packages.

## Recommended order

Package numbers group related work; they are not the execution order. Ordered
by expected benefit relative to cost, with the real prerequisites of each work
package taken from its own file:

1. AP01 and AP02: establish conventions and improve diagnostics.
2. AP07 and AP04: small changes that remove frequent silent mistakes.
3. AP11, then AP16 and AP17: align declaration, binding, and parameter
   keywords with Pascal expectations. AP16.3 and AP17.1 are the one keyword
   migration; AP16.2 prepares it, and AP17.3 completes the intrinsics.
4. AP13: the largest migration. AP13.1 and AP13.2 come first; its later work
   packages unblock AP20, AP21, AP23, and AP26.
5. AP09, AP10, and AP14.
6. AP20, then AP03 and AP15; AP21 and AP24.
7. AP18, AP19, and AP23.
8. AP26, then AP25 after its reassessment.
9. AP05, AP06, and AP12 once their open decisions are settled. AP06.1 is a
   decision work package and can be prepared at any time.

AP27 is optional and follows AP22. AP28 is retained as editor/LSP work outside
this language plan, for transfer after AP13 and AP24. AP29 is rejected.

Several syntax migrations touch the same sources. Where a later work package
would make an earlier rewrite obsolete, the earlier file names the dependency
so each file is migrated once in its final form; AP03.1 and AP20.1 wait for
AP13.5 for that reason.

## Decision gates

Before implementing a work package:

- Resolve the open decisions its file names, and the open decisions of its
  package README, with the user.
- Inspect its owning modules, file sizes, existing features, and tests.
- List the actual files to create, modify, split, move, or remove.

Distinguish **direction agreed**, **specified**, **implemented**, and
**verified**; documentation alone does not complete implementation. Record
status, decisions, affected files, evidence, checks and results, remaining
blockers, and the next step as described in
[development-process.md](development-process.md).

AP22 is approved only for literals, typed construction, and explicit
conversions (Q14); ordinary calls still require annotations. AP27 retains its
implementation dependency on AP22 and is retained as optional (Q21). Do not
broaden inference indirectly through another package.

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

- Package directories `apNN-*/`: one README plus one file per work package.
- [AP01 reference style](ap01-pascal-conventions/reference-style.md): canonical
  draft examples and the distinction between current and planned forms.
- [development-process.md](development-process.md): branches, merges, status
  tracking, migration rules, required checks.
- [shared-constraints.md](shared-constraints.md): cross-package rules,
  regression checks, and design inspiration.

All 23 questionnaire decisions are recorded in the owning packages as Q01–Q23.
These decisions do not mark implementation complete; remaining implementation
details and explicit decision gates stay in the owning packages.
Implemented behavior remains in the [current handbook](../../pascal/README.md).

The branch `codex/syntax-changes` is a reference only: it carries an earlier,
functional-first redesign with its own stage plan and several implemented
slices. Its alias-only imports and removal of record methods diverge from
the decisions recorded in AP05 and AP06 here; the decisions here win.
Package READMEs note the reusable findings from that branch under
**Reference**.
