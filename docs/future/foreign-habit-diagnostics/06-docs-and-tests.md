# 06 – Docs and tests

Documentation and test strategy for the [foreign-habit diagnostics plan](README.md).

## Handbook

- **`docs/pascal/tools/diagnostics.md`** gets a section "Diagnostics for other-language
  spellings": what the compiler recognizes, the shape of the help text, and the new codes from
  [03-architecture.md](03-architecture.md#diagnostic-codes) if allocated. New codes also get a
  wrong and corrected example in the code catalog of that page, as `codes.rs` requires.
- **`docs/pascal/agent-reference.md`** keeps its habit table. Each package checks that every row
  it touches is backed by a diagnostic, and adds rows for new habits under the
  [maintenance guide](../../pascal/agent-reference-maintenance.md). The table stays the place to
  *learn* the FPAS form; the diagnostics catch the habit when it slips through anyway.
- Language pages under `docs/pascal/language/` stay unchanged; the language does not change.

## Permanent catalog

After P6 the catalog must survive this directory's removal. Proposal: the handbook section above
holds a table "habit → FPAS form → diagnostic code", maintained by
hand, with one row per implemented entry. Rejected and deferred entries are dropped; they live in
git history.

**Q-5 (decided):** the handbook table above is the permanent catalog. Rust test names and the
agent-reference table alone would give humans no single overview.

## Rust tests

Per catalog entry, in the stage that detects it:

| Test | Asserts |
| --- | --- |
| Positive | The probe from [02-catalog.md](02-catalog.md) produces exactly one diagnostic with the expected code, a message naming the habit, the span of the whole construct, and help text containing the FPAS form |
| Negative | Valid FPAS using the same tokens (see [04](04-recovery-and-false-positives.md#ambiguities-to-respect)) produces no diagnostic |
| Recovery | Code after the foreign construct is still checked: a second, unrelated error later in the file is still reported |

Locations follow the existing layout:

- lexer: `crates/fpas-lexer/src/tests/errors/foreign_*.rs`
- parser: `crates/fpas-parser/src/tests/errors/foreign_*.rs`, one file per parser module from
  [03-architecture.md](03-architecture.md#parser)
- sema: `crates/fpas-sema/src/tests/integration/foreign_*.rs`

Existing tests that assert today's generic messages for these inputs are updated, not
duplicated.

## Documentation examples

Selected rows can also be shown as handbook fences with `error=FPnnnn`, checked by the
[documentation example checker](../../pascal/tools/documentation-examples.md). Use this only for
the handbook section above, a few representative cases, not for every entry.

## FPAS tests

None required: the behavior is compile-time diagnostics, covered by Rust tests. The full FPAS
suite (`fpas test tests/`) runs at the end of each package to catch accidental changes to valid
programs.

## Measuring the result

The probe set from [02-catalog.md](02-catalog.md) is kept in `.temp-data/a2-probe/` during the
work. After each package, the count of diagnostics per probe file is recorded in that package's
status note. Target after P5: every implemented entry has exactly one diagnostic.
