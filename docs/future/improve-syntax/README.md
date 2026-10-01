# Functional Pascal language redesign

This is the steering plan for a functional-first language with Pascal/Ada
word-based syntax. It replaces the previous package list and questionnaire
decisions in this directory. The approved redesign includes removing existing
features; compatibility with the old language is not a requirement.

These documents specify the target, not implemented behavior. Every code fragment
is a target-language example, not a claim that the current compiler accepts it.
The [current handbook](../../pascal/README.md) remains authoritative for the
checkout. Rewriting this plan does not implement or verify the language.

## Priorities

1. Make functions, immutable value transformations, generic data, and expressions
   the ordinary way to build programs.
2. Give each concept one canonical form. Remove aliases, hidden resolution, and
   context-specific exceptions rather than adding alternate spellings.
3. Explain necessary differences through general rules: data versus resources,
   values versus actions, lexical scope versus running-task lifetime.
4. Keep mutation, evaluation order, failure, and task ownership predictable.
5. Make source and diagnostics understandable to people and LLMs. There are no
   model benchmarks, model-based acceptance gates, or LLM repair-count criteria.

Pascal and Ada are sources of useful conventions, not compatibility targets.
Keep case-insensitive names, `:=`, `function`, `procedure`, `record`, `case`,
`begin`, and `end`. A historical spelling has no priority over a consistent rule.
Do not import an entire language's feature set or complexity.

## Decision authority

The user approved this direction and delegated the remaining special-case
decisions. The [language contract](language-contract.md) records those decisions;
they do not require another questionnaire before planning each stage. Internal
implementation choices may be made within that contract. A discovered conflict
must be resolved explicitly and recorded, not hidden in a compiler exception.
Changing the agreed direction or adding an excluded feature requires agreement.

This directory describes the target and its implementation mapping. Completing
the stage-1 source audit does not change current grammar, code, source examples,
or normative documentation. Implementation deliveries are tracked separately.

## Stages and completion

This is the only stage-completion index. A checkbox means the whole stage passed
its acceptance checks, including applicable migration, documentation, and tests.
The checklists in each document track work within that stage.

| Done | Stage | Prerequisites | Status |
|------|-------|---------------|--------|
| [x] | [1. Language contract](language-contract.md) | None | Source audit, grammar/owner mapping, test inventory and bounded sequence recorded; no language implementation claimed |
| [ ] | [2. Diagnostics](stages/02-diagnostics.md) | Stage 1 contract | Shared optional locations, expectation details and Rust JSON renderer implemented; CLI/runner stream integration pending |
| [ ] | [3. Syntax and names](stages/03-syntax-and-names.md) | Stages 1 and 2 | Planned |
| [ ] | [4. Functional core](stages/04-functional-core.md) | Stages 1-3; coordinated mutation/purity work from stage 5 | Planned |
| [ ] | [5. Effects and tasks](stages/05-effects-and-tasks.md) | Stage 4 facilities; contract fixed in stage 1 | Planned |
| [ ] | [6. Domain types and contracts](stages/06-domain-types-and-contracts.md) | Stages 4 and 5 | Planned |

Stage numbers describe integration order, not six indivisible commits. Stage 1
is a design/audit deliverable; stages 2-6 are implementation deliverables. Stage 4
and the caller-mutation/purity portions of stage 5 form one coordinated migration: new
bindings and value semantics must not ship alongside old implicit caller-mutation
intrinsics. Neither stage is complete merely because its syntax parses.

The concrete order inside that boundary is: stage 4 callable/type facilities,
stage 5 caller references and purity metadata/checks, then stage 4 binding/default
and consumer migration. Complete the task-scope portion of stage 5 afterward.
This is an integration dependency, not a requirement that each entire stage wait
for the other to finish.

Within stage 4, build callable values and generic data before removing methods
whose replacements need those facilities. Removing events requires callable
fields, Option patterns, and ordinary invocation first. Temporary conversion tools
may bridge the old and new source forms; the delivered language has no legacy mode.

## Checkout baseline

Read-only inspection found the following reusable foundations and differences:

- [Diagnostics](../../../crates/fpas-diagnostics/src/diagnostic.rs) already carry
  codes, severity, messages, help, and spans. Their existing `Fxxxx` identity is
  retained; a cosmetic renumbering is not part of this redesign.
- [Expression parsing](../../../crates/fpas-parser/src/parser/expr/precedence.rs)
  still places logical operators above comparisons and already rejects chained
  comparisons. Preserve useful checks while replacing the precedence rules.
- The [binding documentation](../../pascal/language/basics/variables.md) still
  describes immutable `var` and mutable `mutable var`.
- [Closures](../../pascal/language/functions/closures.md), generic routines,
  record updates, pattern matching, and typed tasks already provide foundations.
  Audit their exact behavior before extending or replacing them.
- [Typed task handles](../../pascal/language/concurrency/task-handles.md) still
  include bare-task inference and explicit task-group APIs. Stage 5 must reconcile
  all spawning APIs with lexical ownership, not only the `go` parser production.

These observations are not evidence that any redesign stage passes. Refresh the
relevant sources and tests when beginning implementation; do not reuse historical
fixture counts or old test results as current verification.

## Working rules

- Start with the smallest coherent slice whose contracts are settled. Inspect
  owning modules, directory structure, file sizes, tests, and callers first.
- List actual files to create, change, move, split, or remove before code edits.
- Record status, evidence, remaining work, and the next step in the owning stage.
  Do not add dates, machine metadata, or a chronological implementation journal.
- Migrate intended application behavior. If an approved semantic change exposes
  a dependency on old behavior, record it and adapt that caller explicitly.
- Update current documentation and grammar only alongside implemented behavior.
- Apply [verification and migration rules](verification.md) to every delivered
  slice. No stage is complete with dangling legacy APIs or stale source templates.

## Deliberately outside this plan

No named routine arguments, methods, automatic receivers, computed properties,
events, binding `is`, implicit detached tasks, derivation, or placeholder syntax.
There is no full effect system, general laziness, automatic partial application,
pipe operator, second lambda notation, user-defined operators, open macros, broad
typeclass system, native test blocks, or library-specific grammar.

Typed placeholders and structured editing belong to separate editor/LSP planning
if requested; they are not optional boxes hidden in this language roadmap.
Implementation languages, target platforms, and package distribution are outside
scope. Existing project manifests and source-adjacent compiled units remain the
project model. The [grammar-notation plan](../iso-14977-grammar.md) is separate:
this redesign does not decide ISO notation or require a generated parser.

## Next step

Stage 1's [source audit](audit/source-map.md),
[executable-test inventory](audit/test-inventory.md), and
[bounded implementation sequence](audit/implementation-sequence.md) are recorded.
Existing tests were inspected, not executed for this documentation-only delivery.

Stage 2's shared diagnostic representation is implemented; see its owning stage
for verification and remaining work. Next, carry structured records through the
project/build and runner boundaries, then expose JSON streams consistently in
check/build/run/test. Do not start the broad syntax migration before that gate.
