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

Completed packages describe current implemented rules and regression coverage.
Examples owned by open packages are draft syntax; mixed examples identify their
pending owners. Check current behavior against the checkout before implementing
an open work package.

## Package status

AP01, AP02, AP03, AP04, AP05, AP06, AP07, AP09, AP10, AP11, AP13, AP14, AP15, AP16,
AP17, AP18, AP19, AP20, AP21, AP22, AP23, AP24, and AP26 are complete, including all
their work packages. Their READMEs and work-package files record implemented behavior,
implementation owners, regression coverage, and any independent follow-ups.
Checkboxes use the [status-tracking rule](development-process.md#status-tracking).

Of the 43 packages, 23 are implemented, AP08/AP25/AP29 are rejected and closed,
AP12 and AP27 are deferred with their proposals retained, AP28 is transferred
to editor/LSP planning, and the 14 new packages AP30–AP43 are Decision pending.
They record the language-consistency audit: concrete defects, underspecified
contracts, context-dependent rules, and redundant public forms. Recording
these plans approves neither their proposed language changes nor implementation.
The next recommended work package is AP30.1, the task result type decision.
Reopening either deferred package requires a new explicit user decision.

Completed APs retain their delivered scope and status. A new package that
reassesses an earlier decision must name and explicitly replace that decision
before changing behavior; it does not retroactively mark the original package
unfinished. AP31 and AP32 are bounded repairs to existing contracts/guidance;
their pending status records selection of future work, not an invented need
for a new language rule to correct a defect.

The Done cell tracks delivery; the Status column tracks the current decision.
Use these labels consistently:

| Status | Meaning |
|--------|---------|
| Complete | Implemented and verified within the package's scope. |
| Agreed direction | The feature direction is accepted; named open details may still block implementation. |
| Decision pending | Whether to pursue the proposal, or a required design choice, is still under consideration; implementation is not approved. |
| Deferred | Deliberately parked with its proposal retained; no work is scheduled and reopening requires a new decision. |
| Rejected; closed | The proposal will not be implemented under this plan; its rationale is retained. |
| Transferred | Work is owned by another linked plan. |

Uncertainty about whether a feature is wanted belongs under `Decision pending`.
If the user deliberately parks that unresolved question, use `Deferred` and
list the unresolved details in the package's Open decisions section. Optional
scope and low priority are qualifiers, not substitutes for a decision status.

Effort estimates are preliminary, based on the design rather than a code audit.
Dependencies name the packages a package needs; the real prerequisites per work
package are listed in each work package file.

| Done | Package | Effort | Depends on | Status |
|------|---------|--------|------------|-----------|
| [x] | [AP01: Pascal conventions](ap01-pascal-conventions/README.md) | Small | None | Complete |
| [x] | [AP02: Structured diagnostics](ap02-structured-diagnostics/README.md) | Small | None | Complete (Q01) |
| [x] | [AP03: Explicit closed-enum cases](ap03-explicit-closed-enum-cases/README.md) | Small | AP02, AP20 | Complete (AP03.1, AP03.2) |
| [x] | [AP04: Discarded function values](ap04-discarded-function-values/README.md) | Small | AP02 | Complete |
| [x] | [AP05: Qualified imports](ap05-qualified-imports/README.md) | Medium | AP01, AP02 | Complete: import aliases and alias-aware editor tooling |
| [x] | [AP06: Fixed dot-call targets](ap06-dot-call-targets/README.md) | Medium | AP09.1, AP17.3 (AP06.3) | Complete (Q05) |
| [x] | [AP07: Boolean rules](ap07-boolean-rules/README.md) | Medium | AP02 | Complete (Q02, Q03) |
| — | [AP08: Comma-separated parameter lists](ap08-comma-separated-parameter-lists/README.md) | — | — | Rejected; closed (Q06) |
| [x] | [AP09: Named arguments](ap09-named-arguments/README.md) | Medium | AP02 | Complete (AP09.1, AP09.2) |
| [x] | [AP10: Typed record construction](ap10-typed-record-construction/README.md) | Medium | AP09 | Complete (AP10.1–AP10.3) |
| [x] | [AP11: Individual declarations](ap11-individual-declarations/README.md) | Medium | AP01 | Complete (Q07) |
| [ ] | [AP12: Callable expressions](ap12-callable-expressions/README.md) | Medium if reopened | AP06; renewed user decision | Deferred; proposal retained, no implementation scheduled |
| [x] | [AP13: Explicit block boundaries](ap13-explicit-block-boundaries/README.md) | Large | AP01, AP02 | Complete (Q08, Q09) |
| [x] | [AP14: Remove computed properties](ap14-remove-computed-properties/README.md) | Small | AP06 | Complete (AP14.1, AP14.2) |
| [x] | [AP15: Remove event declarations](ap15-remove-event-declarations/README.md) | Small | AP20 | Complete (AP15.1, AP15.2) |
| [x] | [AP16: Immutable and mutable bindings](ap16-immutable-and-mutable-bindings/README.md) | Large | AP11 | Complete (AP16.1–AP16.3) |
| [x] | [AP17: Visible caller mutation](ap17-visible-caller-mutation/README.md) | Large | AP09, AP13, AP16, AP06.2 (AP17.3) | Complete |
| [x] | [AP18: Subrange types](ap18-subrange-types/README.md) | Large | AP07, AP16, AP22.1 | Complete (AP18.1, AP18.2) |
| [x] | [AP19: Distinct domain types](ap19-distinct-domain-types/README.md) | Large | AP05, AP16 | Complete (AP19.1, AP19.2) |
| [x] | [AP20: Nested patterns and explicit bindings](ap20-nested-patterns-and-explicit-bindings/README.md) | Large | AP07, AP13, AP16 | Complete (AP20.1–AP20.3) |
| [x] | [AP21: Decision expressions](ap21-decision-expressions/README.md) | Large | AP03, AP07, AP13 | Complete (AP21.1, AP21.2) |
| [x] | [AP22: Limited local inference](ap22-limited-local-inference/README.md) | Large | AP02, AP10, AP16, AP19, AP24 | Complete (Q14) |
| [x] | [AP23: Preconditions and postconditions](ap23-preconditions-and-postconditions/README.md) | Large | AP07, AP13, AP16 | Complete: AP23.1 preconditions; AP23.2 postconditions (Q15–Q17) |
| [x] | [AP24: Generic data structures](ap24-generic-data-structures/README.md) | Very large | AP03, AP10, AP11, AP20 | Complete (AP24.1, AP24.2, AP24.3) |
| — | [AP25: Conservative purity](ap25-conservative-purity/README.md) | — | — | Rejected; closed after AP25.1 reassessment (Q18 superseded) |
| [x] | [AP26: Structured task scopes](ap26-structured-task-scopes/README.md) | Large | AP13, AP17 | Complete (AP26.1–AP26.3; Q19, Q20, Q24–Q27) |
| [ ] | [AP27: Typed placeholders](ap27-typed-placeholders/README.md) | Very large if reopened | AP02, AP22; renewed user decision | Deferred; proposal retained, value and spelling unresolved |
| — | [AP28: Structured editing](ap28-structured-editing/README.md) | — | AP13, AP24 | Tooling scope transferred to the [AP28 editor/LSP plan](ap28-structured-editing/structured-editing.md) (Q23) |
| — | [AP29: Bounded derivation](ap29-bounded-derivation/README.md) | — | — | Rejected; closed (Q22) |
| [ ] | [AP30: Sound task result types](ap30-task-result-types/README.md) | Medium | AP26 | Decision pending; confirmed type-checking defect, handle contract unresolved |
| [ ] | [AP31: Exported record constant defaults](ap31-exported-record-defaults/README.md) | Small–medium | AP10, AP24 | Decision pending; bounded repair to the documented contract |
| [ ] | [AP32: Actionable language guidance](ap32-actionable-language-guidance/README.md) | Small | AP02, AP06, AP13, AP26 | Decision pending; current-behavior corrections |
| [ ] | [AP33: Consistent numeric contracts](ap33-numeric-contracts/README.md) | Large | AP07, AP18, AP19 | Decision pending; arithmetic, literals, conversion and text |
| [ ] | [AP34: Consistent type inference](ap34-consistent-type-inference/README.md) | Large | AP22, AP24 | Decision pending; Q14 reassessment must be explicit |
| [ ] | [AP35: Predictable condition composition](ap35-condition-composition/README.md) | Medium–large | AP20, AP23 | Decision pending; pattern contexts and postcondition binder |
| [ ] | [AP36: Consistent calls and construction](ap36-call-and-construction-conventions/README.md) | Large | AP09, AP10, AP20 | Decision pending; labels, callable values and construction |
| [ ] | [AP37: Uniform mutation visibility](ap37-uniform-mutation/README.md) | Large | AP06, AP17; coordinate AP36.1 | Decision pending; shared native/user receiver model |
| [ ] | [AP38: Routine declaration visibility](ap38-routine-declaration-visibility/README.md) | Medium–large | AP11, AP24 | Decision pending; signature visibility versus initialization |
| [ ] | [AP39: Consistent case coverage](ap39-consistent-case-coverage/README.md) | Large | AP03, AP20, AP21 | Decision pending; coverage and unreachable-arm rules |
| [ ] | [AP40: Explicit task lifetimes and shared state](ap40-explicit-task-lifetimes/README.md) | Large | AP26, AP30 (by work package) | Decision pending; ownership, globals and wait contracts |
| [ ] | [AP41: Canonical qualified names](ap41-canonical-qualified-names/README.md) | Medium | AP05, AP20, AP24 | Decision pending; style versus mandatory qualification |
| [ ] | [AP42: Canonical collection operations](ap42-canonical-collection-operations/README.md) | Medium–large | AP06; coordinate AP33/AP37 decisions | Decision pending; aliases, absence and static sortability |
| [ ] | [AP43: Consistent error handling and parsing APIs](ap43-consistent-error-handling/README.md) | Medium | AP33.3; AP35/AP39 only where required | Decision pending; parser APIs and handling conventions |

A package's Done cell becomes `[x]` only when every work package in its own
checklist is done. This table is the only package-completion index; do not
create a second backlog. Rejected packages use `—` and are closed without
implementation. Work deferred to another plan also uses `—`; its destination
and transfer conditions are recorded in the owning package README. A proposal
retained in this plan but deferred, such as AP12 or AP27, keeps `[ ]`: this means
unimplemented, not scheduled. A completed decision work package does not make
its dependent implementation complete or approved.

## Work package index

| Package | Work packages |
|---------|---------------|
| AP01 | AP01.1 reference examples; AP01.2 plan spelling review |
| AP02 | AP02.1 audit and code scheme; AP02.2 shared record; AP02.3 project/build diagnostics; AP02.4 JSON output; AP02.5 reference; AP02.6 parameter declaration diagnostics |
| AP03 | AP03.1 explicit variants and `is`; AP03.2 rejects closed-enum catch-alls (both complete) |
| AP04 | AP04.1 discard statement; AP04.2 require consumed results |
| AP05 | AP05.1 import aliases; AP05.2 alias-aware tooling |
| AP06 | AP06.1 catalog/rules; AP06.2 consumer preparation; AP06.3 native type operations and removal of duplicate call forms (all complete) |
| AP07 | AP07.1 short-circuit evaluation; AP07.2 `Std.Bits`; AP07.3 logical precedence; AP07.4 boolean-only operators |
| AP09 | AP09.1 routine arguments; AP09.2 variant constructors |
| AP10 | AP10.1 typed construction; AP10.2 migrate literals; AP10.3 remove literals |
| AP11 | AP11.1 order-independent types; AP11.2 one keyword per declaration |
| AP12 | AP12.1 decision to defer complete; AP12.2 retained as an inactive sketch |
| AP13 | AP13.1 reserve keywords; AP13.2 terminators; AP13.3 declaration closers; AP13.4 conditionals and loops; AP13.5 case arms; AP13.6 expression closers |
| AP14 | AP14.1 migrate properties; AP14.2 remove declarations |
| AP15 | AP15.1 optional handler fields; AP15.2 ordinary names and field/method model (both complete) |
| AP16 | AP16.1 computed const; AP16.2 migrate immutable var; AP16.3 keyword switch |
| AP17 | Complete: var parameters, named var arguments, and shared mutation checks for caller-mutating intrinsics |
| AP18 | Complete: AP18.1 declarations and checked conversions; AP18.2 Boolean membership |
| AP19 | Complete: AP19.1 declarations and conversions; AP19.2 comparisons |
| AP20 | AP20.1 explicit bindings; AP20.2 nested patterns; AP20.3 `is` test |
| AP21 | Complete: AP21.1 `if` expressions; AP21.2 `case` expressions |
| AP22 | AP22.1 literals and conversions; AP22.2 typed construction |
| AP23 | Complete: AP23.1 preconditions; AP23.2 postconditions |
| AP24 | Complete: AP24.1 parenthesized type arguments; AP24.2 generic records; AP24.3 generic enums |
| AP25 | AP25.1 reassessment complete: decision to close; AP25.2 and AP25.3 removed |
| AP26 | Complete: AP26.1 scope blocks; AP26.2 failure propagation; AP26.3 handle escape restrictions |
| AP27 | AP27.1 spelling decision and AP27.2 analysis retained as deferred drafts |
| AP30 | AP30.1 task type contract; AP30.2 sound result typing |
| AP31 | AP31.1 evaluated constant defaults in unit interfaces |
| AP32 | AP32.1 valid diagnostic hints; AP32.2 current examples and terminology |
| AP33 | AP33.1 numeric decisions; AP33.2 arithmetic/literals; AP33.3 number text |
| AP34 | AP34.1 inference decision; AP34.2 binding and generic inference |
| AP35 | AP35.1 condition decisions; AP35.2 pattern conditions; AP35.3 postcondition binder |
| AP36 | AP36.1 argument conventions; AP36.2 call/construction rules |
| AP37 | AP37.1 mutation model; AP37.2 receiver mutation |
| AP38 | AP38.1 visibility decision; AP38.2 routine signatures |
| AP39 | AP39.1 coverage decision; AP39.2 coverage checks |
| AP40 | AP40.1 concurrency contract; AP40.2 ownership/shared state; AP40.3 waits |
| AP41 | AP41.1 qualification decision; AP41.2 qualified-name rules |
| AP42 | AP42.1 collection contract; AP42.2 collection operations |
| AP43 | AP43.1 error conventions; AP43.2 error APIs and guidance |

AP08, AP28, and AP29 have no work packages.

## Recommended order

The two completed syntax implementation reviews are recorded in
[reviewed scope](review/reviewed-scope.md), including the 22 AP packages and
work packages already inspected. The [review plan](review/README.md) is empty
and retains the file structure and workflow for the next review. Completed
findings and repair evidence remain in Git history.

Package numbers group related work; they are not the execution order. The
completed foundation is listed above. The order below prioritizes correctness
before language simplification. It is a proposed execution order, not approval
of unresolved syntax or semantics. Independent work may proceed separately;
the work-package files state actual dependencies.

| Stage | Recommended sequence | Result required before moving dependent work forward |
|-------|----------------------|-----------------------------------------------------|
| 1. Correctness and trustworthy guidance | **AP30.1 → AP30.2**; independently AP31.1 and AP32.1/AP32.2 | Sound task result types; exported defaults honor the existing contract; hints and examples show valid current syntax |
| 2. Numeric and inference foundations | AP33.1 → AP33.2/AP33.3; AP34.1 → AP34.2 | One numeric contract and a recorded binding/generic inference rule; AP34 need not wait for arithmetic implementation unless its chosen design requires it |
| 3. Conditions and call composition | AP35.1 → AP35.2/AP35.3; AP36.1 → AP36.2 | Transparent condition grouping, an unambiguous postcondition binder, and consistent argument roles, subject to each decision |
| 4. Mutation, declarations and matching | AP37.1 → AP37.2; AP38.1 → AP38.2; AP39.1 → AP39.2 | A native/user mutation model, safe signature visibility, and a complete coverage matrix; these are independent except for explicitly shared syntax choices |
| 5. Concurrency contracts | AP40.1 → AP40.2/AP40.3 | Explicit ownership/shared-state rules and defined wait priorities; may start after the relevant AP30 work instead of waiting for stages 2–4 |
| 6. Canonical public forms | AP41.1 → AP41.2; AP42.1 → AP42.2; AP43.1 → AP43.2 | Qualified-name policy, reduced collection aliases, consistent absence/parsing APIs, and canonical error-handling guidance |

Start with [AP30.1](ap30-task-result-types/01-type-contract.md): the current
checker accepts an integer task result as a string through a bare `task`
parameter. Decide procedure-task and erased-observation types before fixing
the hole so the repair does not remove legitimate completion operations.
AP31 and AP32 can be selected independently without waiting for that decision.

Within stage 6, AP43.2 requires AP33.3's selected number-text contract. It
requires AP35/AP39 implementation only for changes relying on their new
pattern rules. AP42 must preserve the chosen mutation and numeric ordering
contracts; removing aliases also requires preserving evaluation order and
contract-expression capabilities. Later stages are priority guidance, not
artificial dependencies on every earlier package.

Do not remove constructs solely because they can express similar algorithms:
alias/distinct/subrange types, copy/mutation, propagation/panic, and lexical
scopes/dynamic task groups have different contracts. API or syntax removal
requires a canonical replacement and behavior-preserving migration unless a
semantic change is explicitly selected.

AP25 is rejected and closed after its reassessment and geometry pilot; the
user's decision and rationale are retained in its package. No purity
implementation is scheduled.

AP12 is deferred by user decision. Its proposal and implementation sketch are
retained for possible later reassessment, with no scheduled implementation.
Direct calls such as `MakeAdder(3)(5)` do not fit the preferred readability;
existing callable bindings and record-method chains remain available.
Reopening requires a new explicit decision and an inventory of actual gaps.
AP22 is complete using the diagnostic, binding,
conversion, and generic typed-construction rules.

AP27 is also deferred by user decision. Its benefit does not currently justify
pursuing typed placeholders; the proposal and both work-package drafts remain
for later reassessment. Whether to implement it and its spelling remain open.
AP02 and AP22 are complete, but technical readiness does not override deferral.
Only after renewed approval would
[AP27.1: Placeholder spelling](ap27-typed-placeholders/01-placeholder-spelling-decision.md)
precede AP27.2.

The deferred and closed packages are outside the sequence above. AP28's separate
[editor/LSP plan](ap28-structured-editing/structured-editing.md)
starts by inspecting existing editing facilities and selecting one bounded
operation; AP13 and AP24 satisfy its completed language prerequisites.
AP08, AP25, and AP29 are closed without implementation.

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

AP22 remains approved only for literals, typed construction, and explicit
conversions (Q14), including unambiguous non-empty array/dictionary literals
and enum construction; ordinary calls still require annotations. AP34 may
reassess Q14 through its explicit decision work package, but merely adding
AP34 to this plan does not broaden inference. Deferred AP27
retains its implementation dependency on AP22 and its optional scope (Q21). Do not
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
- [AP01 reference style](ap01-pascal-conventions/reference-style.md): reference
  examples and the distinction between current and planned forms.
- [development-process.md](development-process.md): branches, merges, status
  tracking, migration rules, required checks.
- [shared-constraints.md](shared-constraints.md): cross-package rules,
  regression checks, and design inspiration.

All 27 questionnaire decisions are recorded in the owning packages as Q01–Q27.
These decisions do not mark implementation complete; remaining implementation
details and explicit decision gates stay in the owning packages.
Implemented behavior remains in the [current handbook](../../pascal/README.md).
