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

### Cross-checkout Cargo reuse can mix compiler artifacts

- **Date:** 2026-10-30
- **Kind:** awkward (comparison helpers written in FPAS)
- **Area:** build artifact ownership
- **Task:** Compare a pre-byte-buffer checkout with the current compiler under
  the same release settings.
- **Missing or wrong:** Reusing one Cargo target directory across the two
  checkouts retained old workspace objects. A subsequent successful CLI build
  still lacked `Std.Bytes` while loading the current source library. The native
  workload's type check detected the mismatch before any timing was accepted.
- **Minimal example:** After switching checkout contents with a shared
  `CARGO_TARGET_DIR`, a program importing `Std.Bytes` fails with an unknown-unit
  error although the current intrinsic catalog contains that unit.
- **Fallback:** None. Preserve the valid baseline and measurement harness,
  invalidate all local workspace release packages before rebuilding the other
  version, and verify its actual API before measuring. Keep version-specific
  artifacts isolated when retaining both toolchains.

### Standard-library sidecars invalidate CLI builds

- **Date:** 2026-10-30
- **Kind:** awkward (measurement helpers written in FPAS)
- **Area:** build performance
- **Task:** Check equivalent source workloads before comparing array and native
  byte-buffer implementations with the release benchmark harness.
- **Missing or wrong:** Checking against a source standard library publishes
  adjacent `.fpascu` artifacts. The CLI build script watches the entire `lib`
  directory, so these generated files trigger another CLI compilation even
  though its tracked sources are unchanged. Fat-LTO makes that preparation
  expensive. The owner is `crates/fpas-cli/build.rs` and its library sync.
- **Minimal example:** Check a program importing `Std.Net.Http` with
  `fpas check --std-lib lib example.fpas`, then run
  `cargo build --release -p fpas-cli`. Creating the source-library sidecars
  invalidates the preceding CLI build.
- **Fallback:** None. Finish source-library checks before the release build,
  and use the copied profile library during measurements. Precise source
  watching or separation of generated artifacts remains a build-tooling task.

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
