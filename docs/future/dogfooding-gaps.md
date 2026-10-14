# Dogfooding gaps

Intake list for gaps found while agents write their helper scripts in FPAS instead of Python,
JavaScript, or shell logic. The workflow is defined in [AGENTS.md](../../AGENTS.md#dogfooding).
Compiler panics also follow the entry rule in
[compiler-panic-followups.md](compiler-panic-followups.md).

## Entry format

Add one entry per gap under [Open entries](#open-entries):

```markdown
### <Short title>

- **Date:** YYYY-MM-DD
- **Kind:** blocker (fell back to another language) or awkward (done in FPAS)
- **Area:** language, `Std.*` API, bug, diagnostics, or performance
- **Task:** what the helper script had to do
- **Missing or wrong:** what FPAS could not do or did badly
- **Minimal example:** the smallest FPAS source that shows the gap
- **Fallback:** the language or tool used instead, if any
```

Remove an entry once the gap is fixed and covered by tests and current documentation.

## Open entries

### Chained native calls inside an array index

- **Date:** 2026-10-13
- **Kind:** awkward (done in FPAS)
- **Area:** compiler expression typing
- **Task:** select the final segment of a qualified unit name in a migration helper.
- **Missing or wrong:** indexing a split result with another chained call produces
  FP9001 in the current compiler; see the minimal source and resolution in
  [compiler follow-ups](compiler-panic-followups.md#chained-method-call-inside-an-array-index-loses-expression-typing).
- **Fallback:** none; bind the split result to a local constant before indexing.

### Sorting records by a key

- **Date:** 2026-10-13
- **Kind:** awkward (done in FPAS)
- **Area:** `Std.*` API
- **Task:** apply semantic source edits in descending offset order during AP41 migration.
- **Missing or wrong:** array sorting supports naturally ordered values but has no
  key selector or comparator for records. The helper needed a manual insertion sort.
- **Minimal example:** `type Edit = record Offset: integer; Text: string; end record;`
  with `var Edits: array of Edit`; ordering edits by `Offset` requires an explicit loop.
- **Fallback:** none; the migration helper remains FPAS.
