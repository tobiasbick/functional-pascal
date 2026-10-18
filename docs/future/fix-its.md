# Fix-its as data

Plan for agent wish [A3](agent-wishes.md#a3-fix-its-as-data). Status: **planned, not started.**
Nothing here is implemented; the user-facing contract moves to
[`docs/pascal/tools/diagnostics.md`](../pascal/tools/diagnostics.md) once it is.

The FPAS language does not change. This is diagnostics, CLI and editor tooling only.

## Goal

A diagnostic can carry one or more **fixes**: exact source edits that correct it. Agents read them
from `fpas check --diagnostics json`, `fpas check --fix` applies the safe ones in place, and the
language server offers the same fixes as quick fixes. One producer, three consumers.

Success means:

- Every habit diagnostic from [A2](agent-wishes.md#a2-diagnostics-for-pascal-and-ada-habits) whose
  replacement is fully determined by the source has a machine-applicable fix.
- `fpas check --fix` on a file that contains only such habits produces a file that checks without
  errors, and running it again changes nothing.
- No fix ever turns valid FPAS into different valid FPAS: fixes exist only on error diagnostics
  emitted after the canonical parse or check has failed.

## Current state

- [`Diagnostic`](../../crates/fpas-diagnostics/src/diagnostic.rs) has `span`, `help`, `expected`
  and `found`. `expected`/`found` are also used by type mismatches (`integer` vs `string`), so they
  are not a replacement field and stay as they are.
- [`ForeignHabit`](../../crates/fpas-diagnostics/src/foreign_habit.rs) is constructed at 25 sites in
  the lexer, parser and sema. Its `found`/`replacement` are display text: sema abbreviates with `…`
  (`F := …` → `return …`), builtins use placeholders (`Value`, `X`), and the diagnostic span often
  covers more than the text to replace.
- The language service already has quick fixes with exact edits:
  [`SemanticCodeAction` and `SemanticEdit`](../../crates/fpas-language-service/src/semantic_tools/mod.rs),
  currently only for auto-imports.
- `fpas fmt` already publishes rewritten sources atomically
  ([`cli_fmt/publication.rs`](../../crates/fpas-cli/src/cli_fmt/publication.rs)).

## Design

### Fix data in `fpas-diagnostics`

New file `crates/fpas-diagnostics/src/fix.rs`:

- `Fix { title: String, applicability: Applicability, edits: Vec<FixEdit> }`
- `FixEdit { span: SourceSpan, new_text: String }` — a zero-length span is an insertion.
- `Applicability`, named after rustc:
  - `MachineApplicable` — the edits are correct as written; `--fix` applies them.
  - `NeedsReview` — the edits compile but may change meaning or contain placeholders the author
    must fill in (`TryStrToInt` → `Parse.Int`, an initializer guessed from a type). Shown, never
    applied automatically.

`Diagnostic` gets `fixes: Vec<Fix>` and a `with_fix(fix)` builder. Edits of one fix must not
overlap and must lie in the diagnostic's source (`source_id`). A fix may touch several places, for
example a call rewrite plus a `uses Std.Conv as Conv;` insertion.

`ForeignHabit` keeps its display fields for the help text and gains an optional fix; sites that can
compute exact edits pass them, the rest stay prose-only. Help text and `expected`/`found` do not
change.

### JSON output

Each record gets a `fixes` field after `hint`: an empty array when there are none, otherwise

```json
"fixes":[{"title":"Replace `&&` with `and`","applicability":"machine-applicable",
  "edits":[{"start":{"line":3,"column":9},"end":{"line":3,"column":11},"new_text":"and"}]}]
```

Positions use the same one-based Unicode-scalar coordinates as `location`. This extends the JSON
contract; existing fields keep their order and meaning.

### `fpas check --fix`

- Same inputs as `fpas check`. Only sources of the checked project or the named files are edited,
  never dependencies outside the workspace, `lib/` or generated files.
- Loop: check, apply every `MachineApplicable` fix whose edits do not overlap an edit already chosen
  in this pass, write the files, check again. Stop when a pass applies nothing or after 10 passes.
  Several passes are needed because recovery stops at some habits, and fixing one can reveal the
  next.
- Files are written through the existing atomic publication from `cli_fmt`, moved to a shared
  location in `fpas-cli` instead of duplicated.
- Output: the remaining diagnostics as usual (text or JSON), then one summary line on stderr such
  as `applied 7 fixes in 2 files`. Exit code is that of the final check.
- There is no `--dry-run`: `--diagnostics json` already lists every fix without writing.

### Language server

`code_actions` additionally returns each fix of the triggering diagnostic as a quick fix, converted
to `SemanticCodeAction`. `MachineApplicable` fixes are marked preferred. The auto-import action
stays as it is.

## Work packages

| Package | Content | Depends on |
| --- | --- | --- |
| P1 | `Fix`, `FixEdit`, `Applicability`, `Diagnostic::fixes`; JSON serialization and tests | — |
| P2 | Lexer fixes: operators, strings, comments (`&&`, `!=`, `"text"`, `{ … }`, `-- …`) | P1 |
| P3 | Parser fixes: control-flow ends, empty parameter lists, `const` parameters, declaration keywords, generics | P1 |
| P4 | Sema fixes: `Result :=` / `F :=` → `return`, `Inc`/`Dec`/`Length`/`Low`/`High`, type names, qualified built-ins with `uses` insertion | P1 |
| P5 | `fpas check --fix` with the pass loop and shared atomic publication | P1, one of P2–P4 |
| P6 | Language-server quick fixes from diagnostic fixes | P1 |
| P7 | Docs: diagnostics handbook, CLI reference, agent reference, `fpas-authoring` skill; remove this plan | P2–P6 |

Each producer package classifies its sites first: machine-applicable, needs review, or prose only
(class→record, `try … except`, Ada tasks). The classification goes into the habit table of the
diagnostics handbook as a new column.

## Tests

- P1: unit tests for edit validation and JSON field order, escaping and coordinates (non-ASCII,
  tabs, CRLF).
- P2–P4: per site, a test that the fix applied to the source yields text that checks without that
  diagnostic; one shared helper does apply-and-recheck.
- P5: CLI tests for a multi-habit file reaching zero errors, idempotence of a second run, overlap
  handling, the pass limit, untouched dependency files and the exit code.
- P6: language-service test that a habit diagnostic returns its quick fix.
- An FPAS regression fixture under `tests/` with Delphi- and Ada-style sources and their expected
  fixed output.

## Open questions

- Should `--fix` run `fpas fmt` on changed files afterwards, or only insert canonical text and leave
  formatting to an explicit `fpas fmt`?
- Should `--fix` also apply `NeedsReview` fixes behind an extra flag, or never?
