# Dogfooding gaps

Intake list for gaps found while agents write their helper scripts in FPAS instead of Python,
JavaScript, or shell logic.
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

### Available memory and commit headroom

- **Date:** 2026-10-29
- **Kind:** awkward (helper written in FPAS)
- **Area:** host information API
- **Task:** Reserve memory before launching and while supervising Rust builds
  and tests in `tools/resource-guard.fpas`.
- **Missing or wrong:** FPAS has no native query for available physical memory,
  commit headroom or container memory limits.
- **Minimal example:** Process capture and cancellation are available through
  `Std.Os.Proc`, but memory-budget sampling needs OS-specific data queries.
- **Fallback:** Read Linux `/proc` and cgroup files; use a bounded PowerShell CIM
  query on Windows. Policy and owned-process cancellation remain in FPAS, and
  system settings stay unchanged. A native API remains an idea in
  [hardware-information.md](hardware-information.md).
