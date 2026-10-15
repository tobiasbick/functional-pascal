# Documentation example inventory

Status: historical inventory, 2026-10-15, before implementing
[agent wish S1](agent-wishes.md#s1-compile-every-documentation-example).
The [FPAS documentation checker](../pascal/tools/documentation-examples.md) now
implements validation in the normal test suite. Counts, block ids, line numbers
and follow-up assessments below describe the original snapshot, not the current
annotated handbook. Language behavior was unchanged.

The snapshot is commit `5e9a869c`. The [complete block list](documentation-example-blocks.md)
records every fenced block, its opening line, form, compile-probe result,
existing extraction coverage, and required follow-up.

## Scope and counts

| Measure | Count |
| --- | ---: |
| Tracked Markdown files inspected | 244 |
| Fenced blocks, all languages and areas | 903 |
| Pascal/FPAS fences | 499 |
| Other-language, output, configuration and untagged fences | 404 |
| Markdown files under `docs/pascal/` | 161 |
| Fences under `docs/pascal/`, all tags | 564 |
| Pascal fences under `docs/pascal/` | 493 |
| Pascal fences in the main README | 6 |
| Files containing Pascal fences | 124 |
| Unique Pascal sources, preserving source text and normalizing CRLF | 489 |
| Groups of identical Pascal sources | 8 |

All 499 source fences use the plain `pascal` info string. None declares whether
it should compile, execute, reject, or use a context. There are no `fpas` fences
in this snapshot. Future plans contain no Pascal-tagged source fences and were
not checked as current FPAS. The grammar is an EBNF file, not a Markdown example.

All tracked Markdown paths were enumerated, including repository guides and
agent-support material. Rust examples, shell commands, JSON output, TOML
configuration, directory trees and prose notation remain separate from FPAS
source validation. They are listed so that changing a fence tag cannot hide a
block from the inventory. Inline code spans are outside these counts.

## What the compile probes establish

| Source form | Blocks | Result |
| --- | ---: | --- |
| Complete program or unit | 80 | 73 check in isolation; 7 need individual interpretation below |
| Fragments, signatures and notation | 419 | 112 check with a basic wrapper; 307 need additional context or explicit classification |
| Total Pascal fences | 499 | Every fence received a compile probe |

The 80 complete sources comprise 71 programs and 9 units. Units were checked
through a temporary library manifest. All six main-README programs check in
isolation.

The fragment probes try a program header, declarations followed by an empty
body, a statement body, explicit imports followed by a body, or a discarded
expression where appropriate. They do not invent missing imports, declarations,
routine bodies, test data or expected error types. A probe failure therefore
does **not** establish that a fragment is incorrect.

The [block list](documentation-example-blocks.md) uses these probe labels:

- `literal-pass`: the complete source checks unchanged, with a library manifest
  for a unit.
- `literal-fail`: the complete source does not check as an isolated source.
- `wrapper-pass`: a basic wrapper makes the unchanged fragment check.
- `needs-context`: the basic wrappers did not suffice; the note identifies the
  first useful diagnostic or an individually reviewed requirement.
- `not applicable`: another language, configuration, output or notation.

Checks used the compiler from the inventoried tree, `fpas check --std-lib lib`,
with a 20-second process bound per attempt and at most four concurrent probes.
No attempt timed out or produced an internal compiler error. Extracted examples
were not executed. Raw sources, wrapper attempts and diagnostics were retained
in ignored scratch files under `.temp-data/s1-documentation-inventory/`.

## The seven isolated failures

| Block | Source | Interpretation | Required action |
| --- | --- | --- | --- |
| B314 | [Compact reference](../pascal/agent-reference.md), fence at line 1010 | Program imports the companion `Reference.Values` unit; isolated FP4115 | Preserve its existing project context; the reference test already checks and runs it |
| B646 | [Project examples](../pascal/program-structure/projects.md), line 193 | Program imports the documented `MyApp.Math` unit; isolated FP4115 | Keep its adjacent manifest and unit as one example project |
| B652 | [Project examples](../pascal/program-structure/projects.md), line 275 | Program depends on the documented `Acme.Math` library; isolated FP4115 | Keep both manifests and the library unit in the test context |
| B654 | [Units](../pascal/program-structure/units.md), line 47 | Program imports the earlier `MyApp.Utils` unit; isolated FP4115 | Add an explicit association with that companion unit |
| B709 | [Std.Fs](../pascal/std/host/fs.md), line 5 | **Documentation defect:** `go Fs.ReadText(...)` has no owning `scope`; FP3038 | Add an owning scope when repairing this example; execution also needs an input-file fixture |
| B843 | [Diagnostics](../pascal/tools/diagnostics.md), line 50 | **Intentional rejection:** unqualified `ReadText`; expected FP3043 observed | Mark as a compile-negative example, not a broken positive example |
| B863 | [Formatter style](../pascal/tools/fmt-style.md), line 499 | Formatter golden with illustrative `MyApp.Very.Long.Namespace.*` imports; isolated FP4115 | Retain formatter validation; semantic validation needs explicit fixture units |

The two project examples and the reference example already have permanent tests
that use their actual Markdown and the appropriate project context. Their
isolated failures do not invalidate that existing coverage. The `Std.Fs` defect
is recorded here and remains unfixed in this inventory change.

## Contexts the runner will need

| Context | Examples | What must be supplied |
| --- | --- | --- |
| Statement or expression | [String operations](../pascal/language/types/string/case-trim.md), [operators](../pascal/language/basics/operators.md) | Program/body wrapper, explicit imports, enclosing values where referenced |
| Declarations plus usage | [Array SortBy](../pascal/language/types/array/basics.md), fence at line 65; [array callbacks](../pascal/language/types/array/higher-order.md) | Place types/routines before `begin`, and usage inside the body |
| Earlier definitions | [Record methods](../pascal/language/types/record-methods.md), [distinct types](../pascal/language/types/distinct-types.md), [enum patterns](../pascal/language/pattern-matching/enum-patterns.md) | Select the relevant earlier types/helpers; do not concatenate the whole page with repeated definitions |
| Enclosing routine or case | [TUI application](../pascal/std/tui/application.md), [try](../pascal/language/error-handling/try.md) | Correct parameters, return type, message variants and enclosing control flow |
| Multiple files and manifests | [Projects](../pascal/program-structure/projects.md), [units](../pascal/program-structure/units.md), [test hooks](../pascal/std/testing/test.md) | Companion sources, project kind, dependencies and any hook declarations |
| Signature excerpt | [Json](../pascal/std/text/json.md), [Toml](../pascal/std/text/toml.md), [TUI callback signatures](../pascal/std/tui/application.md) | Declaration/type context or comparison with the actual API; missing bodies are intentional |
| Formatter fixture | [Long imports](../pascal/tools/fmt-style.md), line 499 | Formatter check; explicit companion units if semantic checks are added |

The six explicitly reviewed signature excerpts are B787, B789, B802, B804,
B806 and B810. B808 also mixes a callback signature with application setup.
These require a declared context; treating every routine signature as a complete
function would produce false errors.

Three Pascal-tagged blocks are lists/notation rather than one statement list:
B428 ([postfix suffixes](../pascal/language/functions/postfix-chaining.md), line 34),
B555 ([record construction calls](../pascal/language/types/record-methods.md), line 194),
and B817 ([TUI element variants](../pascal/std/tui/elements.md), line 5).
They should become explicitly identified notation or separately checked
expressions with a real context.

The four untagged handbook fences are a keyword list, record-update syntax
notation, and two project directory trees. None conceals an additional complete
FPAS program. The record-update notation is B558 at line 7 of
[record-update.md](../pascal/language/types/record-update.md).

## Negative and mixed examples

Four fences are compile-negative examples; five more combine valid and rejected
source. Expected codes below come from the documented contract. Only B843,
B397 and B566 reached their relevant semantic errors during the basic probes;
the other negative fragments still need their missing context before their
intended diagnostics can be tested.

| Block | Source | Intent / expected code | Preparation |
| --- | --- | --- | --- |
| B397 | [Closures](../pascal/language/functions/closures.md), line 137 | Valid immutable capture, then two rejected task-bound captures; FP3016 | Split into three cases |
| B433 | [Var parameters](../pascal/language/functions/var-parameters.md), line 65 | Rejected array aliasing, then suggested alternative; FP3029 | Supply helpers/array/indices and separate the two calls |
| B438 | [Exhaustiveness](../pascal/language/pattern-matching/exhaustiveness.md), line 24 | Missing enum variant; FP3011 | Supply selector and imports; separate declarations from the body |
| B443 | Same page, line 92 | Missing Result branch followed by the correction; FP3011 | Split the wrong and corrected cases; supply the Result value |
| B444 | Same page, line 113 | Missing nested `Ok(None)`; FP3011 | Supply the nested type, selector and helpers |
| B445 | Same page, line 131 | Unreachable `Some(0)` after `Some(_)`; FP3033 | Supply selector and helpers |
| B520 | [Distinct types](../pascal/language/types/distinct-types.md), line 66 | Valid wrapped argument and two nominal-type errors; FP3006 | Split calls; supply distinct declarations and imports |
| B566 | [Records](../pascal/language/types/records.md), line 89 | Valid alias assignment followed by incompatible record assignment; FP3006 | Separate valid and rejected initializers |
| B843 | [Diagnostics](../pascal/tools/diagnostics.md), line 50 | Unqualified imported name; FP3043 | Add an expected-error annotation |

The [panic example](../pascal/language/error-handling/panic.md), B386 at line 9,
is a different case: it compiles with a program header and deliberately fails
when run. A runner must distinguish compile-negative examples from expected
runtime failures. It must also distinguish commented-out invalid examples, such
as B541 in [generics.md](../pascal/language/types/generics.md), from active source.

## Existing permanent coverage

This counts tests that extract the actual Markdown. Similar standalone fixtures
do not protect a later edit to the documentation and are not counted here.

| Strongest identified extraction coverage per Pascal fence | Blocks |
| --- | ---: |
| Compact-reference project checks and program execution | 13 |
| Selected introductory/formatter program checks and execution | 4 |
| Generic formatter fragment with supplied context, checked and executed | 1 |
| Multi-file project examples, checked | 4 |
| Parser-only extraction | 59 |
| Formatter-only extraction | 1 |
| No permanent extraction test identified | 417 |
| Total | 499 |

Thus **22** fences have identified semantic/project extraction checks, **60**
have only parser/formatter extraction checks, and **417** have no identified
permanent extraction coverage. The compile probes in this inventory are additional
one-time evidence; they do not establish an ongoing test gate.

Relevant existing owners:

- [CLI compact-reference test](../../crates/fpas-cli/src/main_tests/examples/agent_reference.rs)
  checks all 13 fences as projects and executes the 12 programs.
- [Introductory handbook tests](../../crates/fpas-cli/src/main_tests/examples/introductory_handbook.rs)
  check/run the keyword demo and the minimal formatter demo.
- [Formatter control-flow tests](../../crates/fpas-cli/src/main_tests/examples/formatter_handbook.rs)
  check/run the complete control-flow program.
- [Formatter generic-context tests](../../crates/fpas-cli/src/main_tests/examples/formatter_generics.rs)
  check/run the other-types fragment together with its documented generic signature.
- [Project example tests](../../crates/fpas-cli/src/main_tests/projects/documented_examples.rs)
  check/run the first-program example and check both documented multi-file projects.
- [Parser documentation tests](../../crates/fpas-parser/tests/documentation/examples.rs),
  [generic examples](../../crates/fpas-parser/tests/documentation/generics.rs) and
  [declaration examples](../../crates/fpas-parser/tests/documentation/declarations.rs)
  own the parser-only cases.
- [Long-import formatter tests](../../crates/fpas-cli/src/main_tests/fmt/handbook_imports.rs)
  own B863's formatting contract.

The diagnostics catalog also contains inline wrong/corrected pairs.
[Correction tests](../../crates/fpas-parser/tests/documentation/corrections.rs)
extract eight parser pairs, but those inline examples are outside the 499-fence
count. Keyword-table and diagnostic-code-table checks likewise are not semantic
checks of all source examples on their pages.

## Execution requires a separate decision

Successful compilation does not make unattended execution appropriate.
[Console input](../pascal/std/console/input.md), [keyboard](../pascal/std/console/keyboard.md),
[TUI hosting](../pascal/std/tui/application.md), an
[infinite loop](../pascal/language/control-flow/while-repeat.md), host filesystem
operations, process launching, time, random values, and networking need explicit
fixtures or a compile-only policy. The inventory does not assume execution is safe
or deterministic from a page name or a successful check.

## Recommended next implementation order

1. Fix the confirmed `Std.Fs` scope defect and give S1 explicit per-fence
   validation metadata and named contexts. Keep the `pascal` highlighting tag.
2. Adopt the 80 complete sources, preserving companion projects and the existing
   A1 tests. Treat the documented negative example and formatter fixture explicitly.
3. Adopt the 112 fragments already covered by basic wrappers, using declared
   wrappers rather than guessing context during tests.
4. Add reusable import, declaration/body, and companion-source contexts for the
   remaining fragments. Start with string/array/collection and simple Std pages;
   add TUI and other application-specific contexts afterwards.
5. Split the five mixed fences, provide negative contexts, and assert the intended
   diagnostic code instead of accepting any compilation failure.
6. Make signature/notation handling explicit. Add execution only for opted-in
   examples with assertions, expected output or a controlled fixture.

Keep inventory, context migration and the permanent runner reviewable as separate
changes. This document authorizes none of those follow-up implementations by itself.

## Validation of this inventory

- All 244 tracked Markdown files were scanned; all 903 fences were closed.
- All 499 Pascal fences received bounded compile probes: 73 literal passes,
  7 literal failures, 112 wrapper passes and 307 unresolved-context results.
- The seven literal failures and the active negative/mixed examples above were
  individually reviewed against their source and surrounding explanation.
- Untagged handbook fences and existing extraction-test owners were inspected.
- All 903 inventory rows and their opening-fence positions were verified against
  the source documents. All 260 local links in the two inventory documents and
  the updated wish list/index resolve to existing targets. An independent fence
  count confirms the 499 Pascal blocks; `git diff --check` passes.
- No Rust tests were rerun for this documentation-only inventory. Existing test
  coverage was identified by inspecting the extraction-test owners. No Rust or
  language implementation changed.
