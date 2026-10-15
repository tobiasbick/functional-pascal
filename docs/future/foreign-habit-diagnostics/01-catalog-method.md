# 01 – Catalog method

How catalog entries are found, recorded and kept current. Part of the
[foreign-habit diagnostics plan](README.md).

## Sources

1. **Seed list (done in draft).** Every row of the habit table in the
   [compact language reference](../../pascal/agent-reference.md#1-translate-habits-before-writing),
   plus well-known Delphi, Free Pascal, Ada and C-family spellings. Each was turned into a minimal
   program and checked with the current compiler; the result is [02-catalog.md](02-catalog.md).
2. **Blind authoring experiment.** An agent writes programs without reading the FPAS docs, only
   told "a Pascal-family language with Ada-style block endings". Every rejected construct becomes a
   candidate entry.
3. **Ongoing intake.** Habits observed during normal work (dogfooding, tests, examples).

## Blind authoring experiment

### Tasks

Ten short tasks, each touching a different area, so the habits are spread across the grammar:

| # | Task | Areas exercised |
| --- | --- | --- |
| 1 | Sum and average of an integer array | declarations, loops, integer/real mixing |
| 2 | Word count of a string | string indexing, `Length`, chars |
| 3 | Point record with a distance function | records, construction, methods |
| 4 | Parse a number from text and report errors | conversion, error handling |
| 5 | Counter procedure that increments a caller variable | `var` parameters, `Inc` |
| 6 | Shape enum with area by `case` | enums, case, variant data |
| 7 | Linked collection of names (list/dictionary) | generics, `nil`, collections |
| 8 | A unit exporting two functions and a program using it | units, visibility, `uses` |
| 9 | Two tasks communicating through a channel | concurrency |
| 10 | Read a file and print its lines | `Std.Fs`, console output |

### Procedure

1. Run each task twice: once with the instruction "write Delphi-style", once "write Ada-style".
   This yields habits from both families deliberately.
2. Store each attempt under `.temp-data/a2-blind/<task>-<style>.fpas`.
3. Check each file with `bin/fpas check --diagnostics json`.
4. A small FPAS helper script in `.temp-data/` (dogfooding rule from `AGENTS.md`) collects all
   diagnostics into one table: file, line, code, message, source line.
5. Group the rows by habit by hand. Several diagnostics caused by one habit count as one entry
   with a cascade note.
6. Add new entries to [02-catalog.md](02-catalog.md) with origin, current output and proposed hint.

The experiment records only what the compiler said; it does not fix the programs.

## Entry format

Each catalog entry has:

| Field | Meaning |
| --- | --- |
| ID | `H-<area>-<nn>`, stable once assigned |
| Habit | The foreign spelling, minimal |
| Origin | Delphi, FPC, Ada, C-family, or several |
| Today | Code and first message of the current output, plus cascade count |
| Rating | `good` (already helpful), `partial` (right place, weak or misleading hint), `poor` (generic or wrong) |
| Proposed | Proposed message and FPAS form |
| Stage | Lexer, parser or sema, where the detection belongs |
| Package | Rollout package from [05-rollout.md](05-rollout.md), or `rejected` / `deferred` with reason |

## Ongoing intake

Until this plan is complete, new habits go directly into [02-catalog.md](02-catalog.md). After
completion the catalog moves to its permanent location decided in
[06-docs-and-tests.md](06-docs-and-tests.md#permanent-catalog), and new habits are added there
together with their diagnostic.

## Decisions

- **Q-1 (decided).** The experiment runs twice: a short version (tasks 1–5, Delphi style) before
  package P1 to order the packages by frequency, and the full version in P6 to measure the result.
