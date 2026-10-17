# Foreign-habit diagnostics (A2)

Implementation plan for wish [A2](../agent-wishes.md#a2-diagnostics-for-pascal-and-ada-habits):
the compiler recognizes Delphi, Free Pascal, Ada and common C-family spellings and answers with
the FPAS form, for example "FPAS writes this as: …".

**Status: in progress.** P0 (structure) and E1 (short experiment) are done; P1 is next,
see [05-rollout.md](05-rollout.md#package-status). All open questions (Q-1 to Q-5) are decided in
their files. The user approved this README; the other files may still be revised before the
package that implements them starts.

## Goal

An author who writes a familiar non-FPAS spelling gets **one** diagnostic that

1. names the habit it recognized (for example "Delphi `Result :=`"),
2. shows the FPAS spelling with a concrete example, and
3. does not drown in follow-up errors caused by the same mistake.

Success is measurable: every catalog entry marked for implementation has a Rust test that checks
the code, the first message, the help text and the absence of cascades (see
[06-docs-and-tests.md](06-docs-and-tests.md)).

## Scope

- Lexer, parser and semantic diagnostics for foreign spellings that FPAS rejects today.
- Consolidation of the existing scattered habit hints into a small number of thematic modules.
- Handbook documentation of the diagnostics and alignment with the habit table in the
  [compact language reference](../../pascal/agent-reference.md#1-translate-habits-before-writing).

## Out of scope

- **No language change.** A foreign spelling is recognized and rejected, never accepted. No
  aliases, no compatibility modes. Principle
  [A5](../agent-wishes.md#a5-a-principle-for-language-decisions) applies.
- **No machine-applicable fixes.** Structured replacements and `fpas check --fix` are wish
  [A3](../agent-wishes.md#a3-fix-its-as-data). This plan only keeps the hint construction in a
  shape A3 can extend later (see [03-architecture.md](03-architecture.md)).
- No change to the JSON diagnostic contract beyond new codes and better `help` text.

## Assumptions

Confirmed by the user on 2026-10-15.

- **A-1. Codes.** Where a diagnostic code for the underlying error already exists, the habit hint
  refines its message and `help` instead of introducing a new code. New codes are allocated only
  for habits whose current error is a misleading generic code (for example `FP1001` for `@`).
  See [03-architecture.md](03-architecture.md#diagnostic-codes).
- **A-2. Catalog source.** A seed catalog built from known habits and a probe run of the current
  compiler comes first ([02-catalog.md](02-catalog.md)). A blind authoring experiment
  ([01-catalog-method.md](01-catalog-method.md)) extends it afterwards and remains the ongoing
  intake.
- **A-3. Wording.** Help text names the foreign origin only when it helps the reader ("Delphi
  `Result :=`"); it always shows the FPAS form.

## Files and review order

| Order | File | Content | Status |
| --- | --- | --- | --- |
| 1 | [README.md](README.md) | Goal, scope, assumptions, order | approved |
| 2 | [01-catalog-method.md](01-catalog-method.md) | How the catalog is collected and kept up to date | ready |
| 3 | [02-catalog.md](02-catalog.md) | Seed catalog with today's actual compiler output | ready |
| 4 | [03-architecture.md](03-architecture.md) | Modules, consolidation, codes, hint format | ready |
| 5 | [04-recovery-and-false-positives.md](04-recovery-and-false-positives.md) | Detection without cascades or false alarms | ready |
| 6 | [05-rollout.md](05-rollout.md) | Work packages with files, tests and docs | ready |
| 7 | [06-docs-and-tests.md](06-docs-and-tests.md) | Handbook pages and test strategy | ready |

## Definition of done for the whole plan

- Every catalog entry is either implemented, explicitly rejected with a reason, or deferred.
- Habit hints live in the modules defined in [03-architecture.md](03-architecture.md); no
  scattered duplicates remain.
- `cargo test --workspace` and `fpas test tests/` pass; the handbook describes the diagnostics.
- This directory is removed and A2 in [agent-wishes.md](../agent-wishes.md) is marked
  implemented, with the permanent catalog location linked. The intake from
  [01-catalog-method.md](01-catalog-method.md#ongoing-intake) moves to its permanent home.
