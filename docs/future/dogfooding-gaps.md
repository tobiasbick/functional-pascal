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

No entries are currently open.
