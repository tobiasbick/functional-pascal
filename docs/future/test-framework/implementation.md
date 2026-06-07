# Implementation plan (contributor guide)

Step-by-step tasks to implement the FPAS test framework. Read [`README.md`](README.md) for goals and phase overview.

**How to track:** mark tasks `- [x]` when complete; leave `- [ ]` open. Update the [progress summary](#progress-summary) counts and phase headers when a section is finished. Tick phase gates in [`README.md`](README.md) when all verification items for that phase pass.

**Spec drafts:** [`std-test.md`](std-test.md) · [`runner.md`](runner.md) · [`scripted-input.md`](scripted-input.md)

**Verify after each phase:** `cargo fmt`, `cargo build --workspace`, `cargo test --workspace`

---

## Progress summary

| Phase | Tasks | Done | Verification |
|-------|-------|------|--------------|
| [1 — Std.Test](#phase-1--stdtest-assertions) | 35 | 0 | [§ 1.8](#18-phase-1-verification) |
| [2 — fpas test](#phase-2--fpas-test-runner) | 22 | 0 | [§ 2.7](#27-phase-2-verification) |
| [3 — Scripted input](#phase-3--scripted-consoletui-input) | 13 | 0 | [§ 3.4](#34-phase-3-verification) |
| [4 — Graph headless](#phase-4--graph-headless-tests) | 9 | 0 | [§ 4.4](#44-phase-4-verification) |
| [5 — Test projects](#phase-5--test-projects-and-workspace) | 8 | 0 | [§ 5.4](#54-phase-5-verification) |
| [6 — Ergonomics](#phase-6--quality-and-ergonomics) | 7 | 0 | — |

_Update the **Done** column as you check off tasks above._

---

## Cross-cutting integration map

Every new `Std.Test` symbol touches this chain (same pattern as `Std.Env`, `Std.Proc`, …):

```text
docs/pascal/std/test.md
        ↓
fpas-std          unit name, symbols, runtime impl
        ↓
fpas-bytecode     TestIntrinsic enum + Intrinsic::Test(...)
        ↓
fpas-sema         register_std_test in std_registry/loaded/
        ↓
fpas-compiler     compile_test_call in std_calls/
        ↓
fpas-std or fpas-vm   run intrinsic (assert → failure + halt)
        ↓
fpas-compiler tests   compile_and_run / compile_run_err
```

Runner-only work (Phase 2+) stays in `fpas-cli` and reuses `fpas-project` + existing `Vm` APIs — no new bytecode.

Reference: [`docs/pascal/std/README.md`](../../pascal/std/README.md) § Shared implementation touchpoints.

---

## Phase 1 — `Std.Test` assertions

**Goal:** `fpas examples/pascal/test/assert_basics.fpas` runs; failed assert exits non-zero with a readable message.

**Phase complete when:** all tasks below are `[x]` and [§ 1.8 verification](#18-phase-1-verification) passes.

### 1.1 Specification

- [ ] **1.1.1** — `docs/pascal/std/test.md` — **CREATE** canonical user spec from [`std-test.md`](std-test.md)
- [ ] **1.1.2** — `docs/pascal/std/README.md` — **MODIFY** add `Std.Test` index entry
- [ ] **1.1.3** — `docs/pascal/11-stdlib.md` — **MODIFY** list `Std.Test` if that file enumerates units

### 1.2 `fpas-std` — unit registry and runtime

- [ ] **1.2.1** — `crates/fpas-std/src/std_units/units.rs` — **MODIFY** add `STD_UNIT_TEST = "Test"`, append to `STD_UNITS_KNOWN`
- [ ] **1.2.2** — `crates/fpas-std/src/std_units/symbols/names.rs` (or equivalent) — **MODIFY** add `STD_TEST_ASSERT_TRUE`, `STD_TEST_ASSERT_FALSE`, `STD_TEST_ASSERT_EQUALS_INT`, `STD_TEST_FAIL`, `STD_TEST_SKIP`
- [ ] **1.2.3** — `crates/fpas-std/src/std_units/symbols/groups.rs` — **MODIFY** add `STD_TEST_SYMBOLS` slice
- [ ] **1.2.4** — `crates/fpas-std/src/std_units/mod.rs` — **MODIFY** `canonical_std_unit_from_tail("test")`, `std_unit_symbols` match arm
- [ ] **1.2.5** — `crates/fpas-std/src/test/mod.rs` — **CREATE** module root
- [ ] **1.2.6** — `crates/fpas-std/src/test/state.rs` — **CREATE** `TestFailure { message, location }`; thread-local or pass-through via VM
- [ ] **1.2.7** — `crates/fpas-std/src/test/assert.rs` — **CREATE** `assert_true`, `assert_false`, `assert_equals_integer`, `fail`, `skip` → `Result<(), StdError>`
- [ ] **1.2.8** — `crates/fpas-std/src/lib.rs` — **MODIFY** `pub mod test;`

**Assert failure behavior (decision):** intrinsic handler prints diagnostic to stderr (reuse `fpas_diagnostics` style), then returns a dedicated `StdError` that the VM maps to **`Op::Halt`** with process exit code **1**. Do not use uncontrolled `panic!` in Rust.

### 1.3 `fpas-bytecode` — intrinsics

- [ ] **1.3.1** — `crates/fpas-bytecode/src/intrinsic/test.rs` — **CREATE** `TestIntrinsic` enum (discriminants 0..N within Test domain)
- [ ] **1.3.2** — `crates/fpas-bytecode/src/intrinsic/mod.rs` — **MODIFY** `pub mod test;`, `Test(TestIntrinsic)` variant, `From`/`try_from` arms
- [ ] **1.3.3** — `crates/fpas-bytecode/src/lib.rs` — **MODIFY** re-export `TestIntrinsic`
- [ ] **1.3.4** — `crates/fpas-bytecode/src/intrinsic/tests.rs` — **MODIFY** round-trip tests for new intrinsics

Initial variants: `AssertTrue = 0`, `AssertFalse = 1`, `AssertEqualsInteger = 2`, `Fail = 3`, `Skip = 4`.

- [ ] **1.3.5** — (Phase 1b, optional) overload `AssertEquals` for `boolean`, `string`, `real` — separate intrinsics per type

### 1.4 `fpas-sema` — registration

- [ ] **1.4.1** — `crates/fpas-sema/src/std_registry/loaded/test.rs` — **CREATE** `register_std_test` with `define_func` per assert (see `env.rs`)
- [ ] **1.4.2** — `crates/fpas-sema/src/std_registry/loaded/mod.rs` — **MODIFY** `mod test;`, match arm `STD_UNIT_TEST => test::register_std_test(checker)`
- [ ] **1.4.3** — `crates/fpas-sema/src/tests/` — **CREATE** `check_ok` fixtures using `uses Std.Test`

### 1.5 `fpas-compiler` — lowering

- [ ] **1.5.1** — `crates/fpas-compiler/src/compiler/std_calls/test.rs` — **CREATE** `compile_test_call` (mirror `env.rs`)
- [ ] **1.5.2** — `crates/fpas-compiler/src/compiler/std_calls/mod.rs` — **MODIFY** `mod test;`, call `compile_test_call` from `compile_std_library_call`
- [ ] **1.5.3** — `crates/fpas-compiler/src/tests/std_library/test.rs` — **CREATE** pass/fail/skip integration tests
- [ ] **1.5.4** — `crates/fpas-compiler/src/tests/std_library/mod.rs` — **MODIFY** `mod test;`

### 1.6 `fpas-std` / `fpas-vm` — intrinsic dispatch

- [ ] **1.6.1** — `crates/fpas-std/src/intrinsics.rs` — **MODIFY** route `Intrinsic::Test(...)` to `crate::test::run_test_intrinsic`
- [ ] **1.6.2** — `crates/fpas-std/src/test/intrinsic.rs` — **CREATE** pop stack args, call assert helpers
- [ ] **1.6.3** — `crates/fpas-vm/src/vm/execute/mod.rs` — **MODIFY** if needed: map test `StdError` to halt with exit code 1
- [ ] **1.6.4** — `crates/fpas-vm/src/lib.rs` — **MODIFY** (optional) `pub fn test_failure_for_tests(&self)` if runner needs introspection

**Skip behavior:** `Skip` sets skipped flag, prints optional message, **`Halt` with exit 0**.

### 1.7 Examples and CLI smoke

- [ ] **1.7.1** — `examples/pascal/test/assert_basics.fpas` — **CREATE**
- [ ] **1.7.2** — `examples/pascal/test/assert_fail_demo.fpas` — **CREATE** documents expected failure (manual / doc only, not in CI allowlist)
- [ ] **1.7.3** — `examples/README.md` — **MODIFY** mention `examples/pascal/test/`
- [ ] **1.7.4** — `crates/fpas-cli/src/main_tests/examples.rs` — **MODIFY** add `assert_basics.fpas` to `NON_INTERACTIVE_EXAMPLES`

### 1.8 Phase 1 verification

- [ ] `uses Std.Test` resolves in sema
- [ ] `AssertEquals(4, 2+2)` compiles and runs
- [ ] Failed assert: message includes line/column hint
- [ ] `Skip('reason')` exits 0
- [ ] `cargo test -p fpas-compiler std_library::test` passes
- [ ] `fpas examples/pascal/test/assert_basics.fpas` exits 0

---

## Phase 2 — `fpas test` runner

**Goal:** `fpas test examples/pascal/test/` discovers `*_test.fpas`, runs each, prints summary.

**Phase complete when:** all tasks below are `[x]` and [§ 2.7 verification](#27-phase-2-verification) passes.

### 2.1 CLI routing

- [ ] **2.1.1** — `crates/fpas-cli/src/cli_input.rs` — **MODIFY** `CliMode::Test`, parse `fpas test`, extend `CLI_HELP`, `ResolvedCli::Test(CliConfig)`
- [ ] **2.1.2** — `crates/fpas-cli/src/cli_run.rs` — **MODIFY** dispatch `ResolvedCli::Test` → `cli_test::test_cli`
- [ ] **2.1.3** — `crates/fpas-cli/src/main.rs` — **MODIFY** `mod cli_test;`
- [ ] **2.1.4** — `crates/fpas-cli/src/cli_test.rs` — **CREATE** main runner logic
- [ ] **2.1.5** — `docs/pascal/10-projects.md` — **MODIFY** document `fpas test` (from [`runner.md`](runner.md))

### 2.2 Discovery

- [ ] **2.2.1** — `crates/fpas-cli/src/cli_test/discover.rs` — **CREATE** given path: single file, directory glob, or project load
- [ ] **2.2.2** — Filter sources: basename ends with `_test.fpas` (case-insensitive)
- [ ] **2.2.3** — Each file must parse as `program` (not bare `unit`) — error with hint if wrong
- [ ] **2.2.4** — `crates/fpas-project` — **MODIFY** only if needed: expose linked source list (prefer existing load API from `cli_check`)

Reuse: `cli_check.rs` (project load), `main_tests/support.rs` (`run_source_and_capture_output`).

### 2.3 Execution loop

- [ ] **2.3.1** — `crates/fpas-cli/src/cli_test/run.rs` — **CREATE** compile, `Vm::new`, `run()`, collect output per test
- [ ] **2.3.2** — `crates/fpas-cli/src/cli_test/report.rs` — **CREATE** `TestResult`, human summary, exit code aggregation
- [ ] **2.3.3** — Exit codes per [`runner.md`](runner.md): 0 ok, 1 assert fail, 2 compile, 3 runtime
- [ ] **2.3.4** — Optional `--fail-fast` (stop after first failure)
- [ ] **2.3.5** — Optional `--list` (print paths only)

### 2.4 Golden stdout (optional in Phase 2)

- [ ] **2.4.1** — Sidecar `*.expect.stdout` next to test file
- [ ] **2.4.2** — `cli_test/run.rs` — compare `vm.output().lines` after successful run
- [ ] **2.4.3** — Mismatch → test failure with diff hint

### 2.5 Tests

- [ ] **2.5.1** — `crates/fpas-cli/src/main_tests/test_runner.rs` — **CREATE** temp dir with 2–3 `*_test.fpas`, invoke `test_cli`
- [ ] **2.5.2** — `crates/fpas-cli/src/main_tests/mod.rs` — **MODIFY** `mod test_runner;`

### 2.6 Examples

- [ ] **2.6.1** — `examples/pascal/test/readln_test.fpas` — **CREATE** (script in Phase 3)
- [ ] **2.6.2** — `examples/pascal/test/readln_test.script.toml` — **CREATE** (Phase 3)
- [ ] **2.6.3** — `examples/pascal/test/tests.fpasprj` — **CREATE** (optional) bundles test sources

### 2.7 Phase 2 verification

- [ ] `fpas test examples/pascal/test/` — all pass
- [ ] One failing test → exit 1, summary shows FAIL line
- [ ] `fpas test` with no args discovers project like `fpas check` (if single `.fpasprj` in cwd)
- [ ] `cargo test -p fpas-cli test_runner` passes

---

## Phase 3 — Scripted console/TUI input

**Goal:** `tui_escape_test.fpas` + `.script.toml` runs without human input.

**Phase complete when:** all tasks below are `[x]` and [§ 3.4 verification](#34-phase-3-verification) passes.

### 3.1 Script parser

- [ ] **3.1.1** — `crates/fpas-cli/Cargo.toml` — **MODIFY** ensure `toml` dependency (if not already)
- [ ] **3.1.2** — `crates/fpas-cli/src/test_script/mod.rs` — **CREATE**
- [ ] **3.1.3** — `crates/fpas-cli/src/test_script/parse.rs` — **CREATE** TOML → `Vec<ScriptEvent>` per [`scripted-input.md`](scripted-input.md)
- [ ] **3.1.4** — `crates/fpas-cli/src/test_script/apply.rs` — **CREATE** `apply_script(vm: &mut Vm, events: &[ScriptEvent])`
- [ ] **3.1.5** — `crates/fpas-cli/src/test_script/console.rs` — **CREATE** map to `ConsoleEvent` / `push_readln_input` / `push_readkey_input`
- [ ] **3.1.6** — Reuse `fpas_std::{ConsoleEvent, ConsoleKeyEvent, key_kind_index, …}` — no duplicate string tables

### 3.2 Runner integration

- [ ] **3.2.1** — `cli_test/run.rs` — before `vm.run()`: if `<stem>.script.toml` exists beside test file, parse and apply
- [ ] **3.2.2** — `cli_input.rs` / `cli_test.rs` — `--script <path>` overrides auto-discovery
- [ ] **3.2.3** — Parse errors → exit 2 with file/line hint

### 3.3 Examples and tests

- [ ] **3.3.1** — `examples/pascal/test/tui_escape_test.fpas` — **CREATE**
- [ ] **3.3.2** — `examples/pascal/test/tui_escape_test.script.toml` — **CREATE**
- [ ] **3.3.3** — `crates/fpas-cli/src/test_script/tests.rs` — **CREATE** unit tests for TOML parse + apply
- [ ] **3.3.4** — `crates/fpas-cli/src/main_tests/test_runner.rs` — **MODIFY** end-to-end TUI test case

### 3.4 Phase 3 verification

- [ ] Escape in script triggers TUI quit handler
- [ ] Mouse `Down`/`Up` at (x,y) reaches `OnMouse` in hosted TUI test
- [ ] Invalid script `type` → clear error before VM start
- [ ] `cargo test -p fpas-cli test_script` passes

---

## Phase 4 — Graph headless tests

**Goal:** graph programs run in CI without native window; scripted graph events work.

**Phase complete when:** all tasks below are `[x]` and [§ 4.4 verification](#44-phase-4-verification) passes.

### 4.1 Headless mode in runner

- [ ] **4.1.1** — `test_script/parse.rs` — **MODIFY** `[config] headless_graph = true`, graph event types
- [ ] **4.1.2** — `test_script/graph.rs` — **CREATE** map to `GraphEvent`, `push_graph_event`
- [ ] **4.1.3** — `cli_test/run.rs` — wrap `vm.run()` in `with_headless_graph_backend_for_tests` when config flag set

Reference: [`crates/fpas-compiler/src/tests/std_library/graph.rs`](../../../crates/fpas-compiler/src/tests/std_library/graph.rs)

### 4.2 Examples

- [ ] **4.2.1** — `examples/pascal/test/graph_smoke_test.fpas` — **CREATE** minimal open/draw/close
- [ ] **4.2.2** — `examples/pascal/test/graph_smoke_test.script.toml` — **CREATE** quit key event

### 4.3 Phase 4b (optional) — pixel assertions

- [ ] **4.3.1** — Evaluate `Std.Test.AssertPixel` vs runner-only Rust hook — defer unless strong need
- [ ] **4.3.2** — If added: expose `last_headless_graph_frame_for_tests` through intrinsic

### 4.4 Phase 4 verification

- [ ] Graph test completes in CI (no window flash)
- [ ] Scripted `graph_key` / `graph_mouse` consumed by `PollEvent` loop
- [ ] Native graph backend not selected when headless flag set

---

## Phase 5 — Test projects and workspace

**Goal:** dedicated test projects; workspace-wide `fpas test`.

**Phase complete when:** all tasks below are `[x]` and [§ 5.4 verification](#54-phase-5-verification) passes.

### 5.1 Project kind

- [ ] **5.1.1** — `crates/fpas-project/src/` — **MODIFY** accept `kind = "test"` in manifest parser
- [ ] **5.1.2** — `test` projects require `main`; reject bare `fpas my-tests.fpasprj` run (only `fpas test`)
- [ ] **5.1.3** — `docs/pascal/10-projects.md` — **MODIFY** document `kind = "test"`
- [ ] **5.1.4** — `docs/rust/project-loading.md` — **MODIFY** contributor notes

### 5.2 Workspace discovery

- [ ] **5.2.1** — `cli_test/discover.rs` — **MODIFY** no path: find workspace, collect `kind = "test"` members
- [ ] **5.2.2** — `[test]` manifest section optional: per-file `script`, `headless_graph` overrides

Example layout:

```text
apps/my-app/
 ├── my-app.fpasprj          kind = "program"
 └── my-app-tests.fpasprj    kind = "test", depends on my-app library units
```

### 5.4 Phase 5 verification

- [ ] `fpas test` at workspace root runs test members
- [ ] `fpas my-app-tests.fpasprj` errors with hint to use `fpas test`
- [ ] `fpas check` still accepts test projects

---

## Phase 6 — Quality and ergonomics

Implement incrementally; no fixed order. Mark each item `[x]` when shipped.

- [ ] **6.1** — `--filter <pat>` in `cli_test/discover.rs` — substring on path
- [ ] **6.2** — `--report json` in `cli_test/report.rs` — CI structured output
- [ ] **6.3** — Parallel test execution in `cli_test/run.rs` — one VM per test; verify graph headless thread safety first
- [ ] **6.4** — Setup/Teardown convention in `Std.Test` + docs — named procedures in test units
- [ ] **6.5** — TUI screen snapshot spike in `fpas-std` TUI session — export cell grid
- [ ] **6.6** — Runner timeout flag for hung interactive tests
- [ ] **6.7** — Property/fuzz for event ordering — Rust only, out of FPAS scope

---

## Dependency graph (implementation order)

```mermaid
flowchart TD
  P1[Phase 1 Std.Test]
  P2[Phase 2 fpas test]
  P3[Phase 3 scripted input]
  P4[Phase 4 graph headless]
  P5[Phase 5 test projects]
  P6[Phase 6 ergonomics]

  P1 --> P2
  P2 --> P3
  P3 --> P4
  P2 --> P5
  P3 --> P6
  P4 --> P6
```

Phases 3 and 5 can overlap after Phase 2. Phase 4 depends on Phase 3 script infrastructure.

---

## Files created by phase (summary)

```text
Phase 1
 docs/pascal/std/test.md
 crates/fpas-std/src/test/{mod,state,assert,intrinsic}.rs
 crates/fpas-bytecode/src/intrinsic/test.rs
 crates/fpas-sema/src/std_registry/loaded/test.rs
 crates/fpas-compiler/src/compiler/std_calls/test.rs
 crates/fpas-compiler/src/tests/std_library/test.rs
 examples/pascal/test/assert_basics.fpas

Phase 2
 crates/fpas-cli/src/cli_test.rs
 crates/fpas-cli/src/cli_test/{discover,run,report}.rs
 crates/fpas-cli/src/main_tests/test_runner.rs

Phase 3
 crates/fpas-cli/src/test_script/{mod,parse,apply,console,tests}.rs
 examples/pascal/test/tui_escape_test.fpas
 examples/pascal/test/tui_escape_test.script.toml

Phase 4
 crates/fpas-cli/src/test_script/graph.rs
 examples/pascal/test/graph_smoke_test.fpas (+ .script.toml)

Phase 5
 (mostly modifications to fpas-project, cli_test/discover, docs)
```

---

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Assert failure indistinguishable from runtime error | Dedicated `TestIntrinsic` + exit code 1; document in runner |
| TUI tests hang without script | Runner timeout flag (Phase 6); Phase 3 docs require script for interactive tests |
| Graph native window in CI | Mandatory `headless_graph` for graph tests in runner |
| Duplicate enum string tables in script parser | Import indices from `fpas-std` only |
| `AssertEquals` overload ambiguity | Separate intrinsics per type in sema |

---

## Related contributor docs

| Document | Use when |
|----------|----------|
| [`docs/rust/project-loading.md`](../../rust/project-loading.md) | Phase 2/5 runner + project load |
| [`docs/pascal/std/README.md`](../../pascal/std/README.md) | Any `Std.*` wiring |
| [`tui-application-framework.md`](../tui-application-framework.md) | Phase 3 TUI test expectations |
| [`examples/README.md`](../../../examples/README.md) | Non-interactive allowlist updates |
