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
AP17, AP18, AP19, AP20, AP21, AP22, AP23, AP24, AP26, AP30, AP31, AP32, AP33, AP34,
AP35, AP36, AP37, AP38, AP39, AP40, AP41, AP42, and AP43 are complete, including all their work packages.
Their READMEs and work-package files record implemented behavior,
implementation owners, regression coverage, and any independent follow-ups.
Checkboxes use the [status-tracking rule](development-process.md#status-tracking).

Of the 46 packages, 37 are implemented, AP08/AP25/AP29 are rejected and closed,
AP12 and AP27 are deferred with their proposals retained, AP28 is transferred
to editor/LSP planning, and AP44–AP46 have agreed directions with delivery pending.
The FPAS 1 stabilization scope is bounded: unambiguous enum values, an explicit
snapshot-iteration contract, and Unicode character-code literals. Broader
[FPAS 2 directions](../fpas2-language-directions.md) are retained separately and
are not scheduled here. AP36 implements its recorded call, construction, and
pattern contracts. AP37 implements its value/InPlace contract, named Pop result,
consumer migration and editor support. AP38 implements scope-wide named routine
signatures, checked initialization, compiled-unit summaries, and editor identity.
AP39 implements one exhaustive case matrix, exact finite coverage, and uniform
duplicate/reachability checks, with consumers and editor guidance aligned.
AP40 implements owned starts, one go expression, ordinary discard, foreign
mutable-state checks, creating-task group cleanup, ordinary Result errors, and
consistent controlled wait priorities, with consumers and tooling migrated.
AP41 implements mandatory alias imports and type-qualified user enum variants,
including semantic migration, diagnostics, cached interfaces, and editor output.
AP42 implements canonical collection forms, Option-valued index searches, and
static stable sorting through Comparable. AP43 implements Result-returning
Int/Real/Bool/HexInt parsers, canonical error-handling roles, selected helper
removals, preserved caller effects, and strict generic try error compatibility. These
packages record the language-consistency audit: concrete defects, underspecified contracts,
context-dependent rules, and redundant public forms. Adding a proposal by itself
does not approve its language changes or implementation.
AP30 and AP31 implement their selected contracts; AP32 corrects diagnostic
hints and language guidance against current implemented behavior. AP33 delivers
explicit numeric compatibility, checked arithmetic, and shared numeric text rules.
AP34 delivers uniform local/contextual inference, including native callbacks and
rejection of unresolved inner sites. AP35 delivers nonbinding Boolean is tests,
case-only pattern bindings, and `ensures (Name):` return binders.
AP36 retains AP09's complete positional-or-named call forms and implements named
callable-value support through visible static signatures. Its delivered contract
preserves both enum construction forms and named record defaults, adds fully
named enum/Option/Result patterns, and gives named Some/Ok/Error forms the payload
label Value. Every pattern field remains explicit; styles cannot mix within a
field list. AP37 retains explicit var arguments, read-only Self and value-producing
transformations; native mutation uses InPlace counterparts. Pop returns Remaining
and Value; PopInPlace updates storage and returns the element. AP38 implements
scope-wide named routine signatures and source-order value initialization,
with conservative compile-time checks for premature direct or indirect use.
AP39 implements always-exhaustive case with one statement/expression matrix and
uniform reachability checks. AP40 delivers its selected concurrency contract;
AP41 delivers its qualification contract; AP42 delivers its canonical collection
contract; AP43 delivers its parsing/handling contract, migration, and tooling.
Reopening either deferred package requires a new explicit user decision.

Completed APs retain their delivered scope and status. A new package that
reassesses an earlier decision must name and explicitly replace that decision
before changing behavior; it does not retroactively mark the original package
unfinished. AP31 delivers a bounded repair to the existing record-default
contract. AP32 delivers guidance corrections without changing syntax or semantics.

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
| [x] | [AP30: Sound task result types](ap30-task-result-types/README.md) | Medium | AP26 | Complete (AP30.1, AP30.2): fixed procedure/function task types |
| [x] | [AP31: Exported record constant defaults](ap31-exported-record-defaults/README.md) | Small–medium | AP10, AP24 | Complete (AP31.1): evaluated constant defaults and source diagnostics |
| [x] | [AP32: Actionable language guidance](ap32-actionable-language-guidance/README.md) | Small | AP02, AP06, AP13, AP26 | Complete: valid hints, examples and terminology |
| [x] | [AP33: Consistent numeric contracts](ap33-numeric-contracts/README.md) | Large | AP07, AP18, AP19 | Complete |
| [x] | [AP34: Consistent type inference](ap34-consistent-type-inference/README.md) | Large | AP22, AP24 | Complete (AP34.1–AP34.2; Q14 restriction replaced) |
| [x] | [AP35: Predictable condition composition](ap35-condition-composition/README.md) | Medium–large | AP20, AP23 | Complete (AP35.1–AP35.3) |
| [x] | [AP36: Consistent calls and construction](ap36-call-and-construction-conventions/README.md) | Large | AP09, AP10, AP20 | Implemented |
| [x] | [AP37: Uniform mutation visibility](ap37-uniform-mutation/README.md) | Large | AP06, AP17, AP36 | Complete (AP37.1–AP37.2; native Push/Pop exception replaced) |
| [x] | [AP38: Routine declaration visibility](ap38-routine-declaration-visibility/README.md) | Medium–large | AP11, AP24 | Complete: scope-wide signatures, checked initialization, compiled-unit summaries, and compiler/editor declaration identity |
| [x] | [AP39: Consistent case coverage](ap39-consistent-case-coverage/README.md) | Large | AP03, AP18, AP19, AP20, AP21 | Complete (AP39.1–AP39.2): shared exhaustive coverage, fixed else matrix, exact intervals, and uniform reachability |
| [x] | [AP40: Explicit task lifetimes and shared state](ap40-explicit-task-lifetimes/README.md) | Large | AP26, AP30; preserve AP37/AP38 | Complete (AP40.1–AP40.3): owned starts, ordinary discard, foreign mutable-state checks, joined owner teardown, ordinary Result errors, and consistent wait priorities |
| [x] | [AP41: Canonical qualified names](ap41-canonical-qualified-names/README.md) | Medium | AP05, AP20, AP24 | Complete: mandatory alias imports, type-qualified user variants, retained reserved Option/Result forms, migrated consumers, and matching diagnostics/editor output |
| [x] | [AP42: Canonical collection operations](ap42-canonical-collection-operations/README.md) | Medium–large | AP06; extend AP24; preserve AP33/AP37 | Complete (AP42.1–AP42.2): canonical forms, Option search and static stable sorting |
| [x] | [AP43: Consistent error handling and parsing APIs](ap43-consistent-error-handling/README.md) | Medium | AP33.3, AP35, AP39; preserve AP34/AP36/AP41/AP42 | Complete (AP43.1–AP43.2): Result parsers, canonical handling roles, selected helper removals, migration and tooling |
| [ ] | [AP44: Unambiguous enum values](ap44-unambiguous-enum-values/README.md) | Small–medium | AP24, AP39, AP41, AP42 | Agreed direction: unique simple-enum values; no backing values in payload enums |
| [ ] | [AP45: Explicit snapshot iteration](ap45-snapshot-iteration/README.md) | Small | AP37 | Agreed direction: document and verify existing array/dictionary iteration behavior |
| [ ] | [AP46: Unicode character-code literals](ap46-unicode-character-literals/README.md) | Small | AP33, AP42 | Agreed direction: accept Unicode scalars in decimal character-code literals |

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
| AP30 | Complete: AP30.1 task type contract; AP30.2 sound result typing |
| AP31 | Complete: AP31.1 evaluated constant defaults in unit interfaces |
| AP32 | Complete: AP32.1 valid diagnostic hints; AP32.2 current examples and terminology |
| AP33 | Complete: AP33.1 numeric decisions; AP33.2 arithmetic/literals; AP33.3 number text |
| AP34 | AP34.1/AP34.2 complete: uniform binding, generic result and callable inference |
| AP35 | Complete: nonbinding Boolean is tests, case-only pattern bindings, and explicit postcondition return binders |
| AP36 | Complete: AP36.1 call/construction/pattern decisions; AP36.2 compiler, tooling, documentation, and regressions |
| AP37 | Complete: value/InPlace pairs, ArrayPopResult with Remaining/Value, migrated consumers and verified storage/editor behavior |
| AP38 | Complete: AP38.1 visibility decision; AP38.2 routine signatures, initialization proof, tooling, and verification |
| AP39 | Complete: AP39.1 coverage decision; AP39.2 shared checks, exact intervals, migration, tooling, documentation, and regressions |
| AP40 | Complete: AP40.1 concurrency decision; AP40.2 ownership/discard/shared state/teardown and migration; AP40.3 controlled wait priorities with retained retrieval/observation and Select case order |
| AP41 | Complete: AP41.1 mandatory qualification decision; AP41.2 compiler enforcement, semantic consumer migration, alias/enum tooling, documentation, and regressions |
| AP42 | Complete: canonical collection forms, Option index searches, shared static ordering, stable sorting, migration and tooling |
| AP43 | Complete: AP43.1 parser/error contract; AP43.2 API enforcement, migration, tooling and guidance |
| AP44 | AP44.1 enum declaration restrictions, consumer migration and identity/coverage regressions |
| AP45 | AP45.1 snapshot-iteration specification and mutation/reassignment regressions |
| AP46 | AP46.1 Unicode scalar lexing, diagnostics, formatter checks and current guidance |

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
| 1. Correctness and trustworthy guidance | Complete: AP30, AP31, and AP32 | Sound task result types and exported constant defaults are delivered; hints and examples show valid current syntax |
| 2. Numeric and inference foundations | Complete: AP33 and AP34 | Numeric contracts and uniform local/contextual inference are delivered; unresolved sites are rejected before lowering |
| 3. Conditions and call composition | Complete: AP35 and AP36 | Nonbinding conditions and consistent complete call/pattern forms are delivered, preserving AP09/AP10 and AP34/AP35 behavior |
| 4. Mutation, declarations and matching | Complete: AP37, AP38, and AP39 | Explicit mutation, scope-wide signatures, checked initialization, and shared exhaustive case coverage are delivered |
| 5. Concurrency contracts | Complete: AP40.1–AP40.3 | Owned starts, ordinary discard, foreign mutable-state checks, joined owner teardown, and consistent controlled wait priorities are delivered; AP30 types/retrieval and Select case order are retained |
| 6. Canonical public forms | Complete: AP41, AP42 and AP43 | Qualification, collections and Result parsers are delivered, with canonical handling roles, migrated consumers, diagnostics and editor guidance |
| 7. FPAS 1 stabilization | Pending: AP44 → AP45 → AP46 | Enum identities agree across equality and case coverage; iteration behavior is explicit and tested; character-code literals use the string scalar model |

The next recommended work package is AP44.1, followed by AP45.1 and AP46.1.
Their prerequisites are complete; this order reflects correctness and
stabilization priority rather than dependencies between the three packages.
AP44 replaces the permission for duplicate simple-enum backing values from
AP42 and tightens AP24's mixed backing/payload declarations; those completed
packages keep their other contracts and status. AP45 freezes existing behavior.
AP46 replaces only the byte-range restriction on character-code literals.
The current handbook and grammar change with delivery, not with planning.

AP12 and AP27 stay deferred and require a renewed explicit decision; AP28 belongs
to editor/LSP planning. Reopening a deferred proposal is a separate decision.
The broader [FPAS 2 directions](../fpas2-language-directions.md) do not extend
this FPAS 1 implementation sequence.

AP43 is complete. Its contract uses
Result-returning Parse.Int/Real/Bool/HexInt, explicit Unwrap for deliberate abort,
nonbinding is tests, and case for payloads/local handling/plain defaults. It removes
the selected parser aliases, aborting Conv parser wrappers, Option/Result variant
predicates, and UnwrapOr while retaining Map/AndThen/OrElse. No selected AP43
language decisions or implementation work remain open. Parser function values,
cached-unit consumers, strict generic error propagation, current references and
editor declarations are implemented and independently reviewed.
AP42 is complete:
string indexing, membership through `in`, the RepeatStr value/InPlace pair,
Option-valued index searches, and static stable sorting through `Comparable`,
including payload-free enums, are implemented and independently reviewed.
No selected AP42 language decisions remain open.
AP41 is complete: all imports require an explicit alias and imported symbols use
that alias; user enum variants carry their type qualifier in every expression and
pattern position. Some/None/Ok/Error remain the reserved language forms. Alias
uniqueness/non-shadowing, local names, explicit type aliases, and native/record
operations retain their rules. Editor auto-imports and completions generate the
same required forms. Compiler checks, semantic migration, documentation, tooling,
cached interfaces, and regressions are delivered and independently reviewed.
AP40 is complete: every start has a lexical scope or explicit group owner,
helpers do not inherit scopes, and go/discard each have one form. Private
dependency summaries and mandatory VM checks reject foreign mutable access.
Owners cancel and join open groups before publishing outcomes; Result errors
remain data. Controlled probes use cancellation/readiness/timeout priority.
AP30's fixed types and single retrieval, completion observation, and Select's
explicit case order remain intact.
AP37 is complete: value/InPlace pairs retain collection value semantics and exact
writable-storage rules. Pop returns ArrayPopResult with Remaining and Value;
PopInPlace returns the removed element. Both retain the empty-array runtime error.
AP38 is complete: named routines are visible throughout their
lexical scope, including mutually recursive siblings; values initialize in
source order. Conservative initialization checks cover calls, callbacks,
captures, and record defaults, including independently compiled units. Compiler
and editor preserve the selected declaration under lexical shadowing. AP39 is
complete: case is exhaustive in both positions; enum/Option/Result,
Boolean, and subrange forbid else, while integer/string require it. Distinct
selectors follow their supported base type. Guards never count as coverage;
duplicates and fully covered labels are errors, while partial overlaps retain
first-match behavior. Exact interval reasoning covers finite subranges without
enumerating values. Diagnostics, compiled-unit checks, consumers, and editor
guidance follow the same matrix; no AP39 contract decision remains open.
AP36 is complete: calls use
visible static labels and written evaluation order, enum construction keeps
both forms, records keep named defaults, and patterns accept complete named
fields. Some/Ok/Error use Value for their named payload. Nested field lists
independently select a complete style; case scope/coverage and nonbinding is
remain unchanged. Compiler, editor, grammar, and handbook describe this behavior.
AP35 is complete: it replaces AP20.3's
binding/context rules and AP23/Q16's binder punctuation with the recorded forms.
AP34's binding and generic inference contracts are complete.
AP33's arithmetic, literal, and number-text contracts are complete.
AP32.1/AP32.2 retain current syntax and semantics
and are complete. AP31.1 preserves documented scalar constant defaults through
independently compiled units and is complete.
AP30 supplies fixed meanings for bare `task` and `task of T`, homogeneous task
arrays, and existing TaskCase/Select for mixed completion observation. Its
implementation, consumers, documentation, and regression coverage are complete.

Within stage 6, AP43.2 uses the completed AP33.3 number-text and AP35/AP39
pattern/case contracts. It preserves parsing interpretation, try compatibility,
panic separation, and receiver/default evaluation during migration. AP42 preserves
the chosen mutation and numeric ordering contracts
and extends AP24's Comparable admissibility to payload-free enums. Removing
aliases also requires preserving evaluation order, meaningful user scalar
invariants, and contract-expression capabilities. Later stages are priority
guidance, not artificial dependencies on every earlier package.

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

AP22 delivered limited inference for literals, typed construction, and explicit
conversions (Q14), including unambiguous non-empty array/dictionary literals
and enum construction. AP34 explicitly replaces its statement-local
initializer whitelist: every fully determined initializer is eligible, with
generic arguments inferred from supplied values before expected types fill
unresolved parameters. The compiler and handbook implement that rule. Deferred AP27
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
