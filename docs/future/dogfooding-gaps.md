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

### Batch debugger worker exits during a TUI test

- **Date:** 2026-10-31
- **Kind:** awkward (diagnosis remained in FPAS)
- **Area:** diagnostics
- **Task:** Inspect the viewport and footer position at a failing headless TUI
  test assertion in the game client.
- **Missing or wrong:** The batch debugger returned `worker_failure` before
  publishing a breakpoint result; the underlying cause was not established.
- **Minimal example:** In the game workspace, run `fpas debug` on
  `tests/client/client-tests.fpasprj` with a breakpoint in
  `tests/client/inventory_window_test.fpas` and request its `Screen.Height`.
- **Fallback:** Temporarily print the FPAS test's surface snapshot and remove
  the diagnostic output after correcting the layout.

### Commit headroom and ancestor memory limits

- **Date:** 2026-10-29
- **Kind:** awkward (helper written in FPAS)
- **Area:** host information API
- **Task:** Reserve memory before launching and while supervising Rust builds
  and tests in `tools/resource-guard.fpas`.
- **Missing or wrong:** Native RAM, swap and backend cgroup information are
  available through `Std.Os.Hardware.Memory`, but `sysinfo` does not expose the
  Windows commit counters or a complete ancestor-limit inventory needed by
  the guard.
- **Minimal example:** `Std.Os.Hardware.Memory().Unwrap().AvailableBytes`
  supplies available host RAM, not system commit headroom.
- **Fallback:** The guard uses the native API for RAM and swap. Windows commit
  sampling keeps its bounded PowerShell CIM query; Linux strict-overcommit
  counters and ancestor cgroup limits keep their file queries. Policy and
  owned-process cancellation remain in FPAS, and system settings stay unchanged.
  See the [hardware API](../pascal/std/host/hardware.md#memory) for its scope.
