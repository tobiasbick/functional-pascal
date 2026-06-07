# FPAS Test Framework

Implementation plan for a JUnit-style testing surface in Functional Pascal: write tests in `.fpas`, run them with `fpas test`, and drive interactive programs (console, TUI, graph) with scripted keyboard and mouse input — without blocking on a real terminal or native window.

**Track progress:** check boxes in this file (overview) and in [`implementation.md`](implementation.md) (detailed tasks). Mark `[x]` when done; leave `[ ]` for open work.

**Principles**

- **Tests are FPAS programs** — assertions and test cases live in the language, not only in Rust integration tests.
- **Reuse existing VM test hooks** — input queues, headless graph backend, and stdout capture already exist; the framework wires them for authors.
- **Deterministic by default** — no real stdin, no crossterm poll, no native graph window in the default test run.
- **No backward compatibility** for experimental APIs (see root `AGENTS.md`).

---

## Progress overview

| Phase | Status | Summary |
|-------|--------|---------|
| **0** Requirements | 🟡 In progress | Planning docs; open decisions pending |
| **1** `Std.Test` | ⬜ Not started | Assertions, intrinsics, example |
| **2** `fpas test` | ⬜ Not started | CLI runner, discovery, summary |
| **3** Scripted input | ⬜ Not started | TOML sidecar, TUI keyboard/mouse |
| **4** Graph headless | ⬜ Not started | Headless backend + graph events |
| **5** Test projects | ⬜ Not started | `kind = "test"`, workspace |
| **6** Ergonomics | ⬜ Not started | Filter, JSON report, snapshots |

Update the **Status** column when a phase is complete (⬜ → ✅). Detailed task checkboxes live in [`implementation.md`](implementation.md).

### Phase gates (mark `[x]` when the whole phase is done)

- [x] **Phase 0** — Planning documents written ([`README.md`](README.md), [`implementation.md`](implementation.md), specs)
- [ ] **Phase 0** — Open decisions reviewed and recorded
- [ ] **Phase 1** — `Std.Test` shippable; `assert_basics.fpas` runs via `fpas`
- [ ] **Phase 2** — `fpas test` discovers and runs `*_test.fpas`
- [ ] **Phase 3** — TUI test with `.script.toml` passes in CI
- [ ] **Phase 4** — Graph smoke test headless in CI
- [ ] **Phase 5** — Workspace + `kind = "test"` supported
- [ ] **Phase 6** — At least filter + JSON report shipped

---

## Problem

Today, FPAS programs are tested almost exclusively from **Rust**:

- `compile_and_run(source)` in `fpas-compiler` tests
- `vm.push_readln_input`, `push_console_event`, `push_graph_event` in `fpas-vm` tests
- A curated allowlist of non-interactive examples in `fpas-cli`

Interactive demos (TUI, graph, key loops) cannot be batch-run safely — see [`examples/README.md`](../../examples/README.md). Authors have no `Std.Test`, no `fpas test`, and no way to express “press Escape, then assert output” purely in FPAS.

---

## Goals

| Goal | Description |
|------|-------------|
| **Assert in FPAS** | `AssertEquals`, `AssertTrue`, `Fail`, etc. via `Std.Test` |
| **Test runner** | `fpas test [<path>]` discovers and runs test programs, reports pass/fail summary, non-zero exit on failure |
| **Scripted I/O** | Optional sidecar file queues keyboard, mouse, resize, paste, and graph events before run |
| **Headless UI** | TUI and graph tests run without real terminal focus or OS window (reuse test mode + headless graph) |
| **Project integration** | Tests colocated with apps/libraries; workspace-wide `fpas test` |

## Non-goals (initial phases)

- Precompiled test artifacts or separate link steps
- Property-based / fuzz testing in FPAS (stay in Rust for now)
- Screenshot or pixel-diff testing of native graph windows
- Mocking filesystem, network, or clock (later `Std.Test` extensions)
- IDE test gutter integration (follows once CLI exists)

---

## Existing infrastructure

Most simulation plumbing is **already implemented** in Rust; the framework exposes it to FPAS authors.

| Capability | Rust API | FPAS surface today |
|------------|----------|-------------------|
| Line input | `Vm::push_readln_input` | `Read`, `ReadLn` |
| Raw keys | `Vm::push_readkey_input` | `ReadKey` |
| Structured keys | `Vm::push_key_event` | `ReadKeyEvent` |
| Terminal events | `Vm::push_console_event` | `ReadEvent`, `PollEvent`, `ReadEventTimeout`, TUI host |
| Graph events | `Vm::push_graph_event` | `Std.Graph.Application.PollEvent` |
| Stdout capture | `Vm::output()` | (Rust-side assertion only) |
| Non-blocking I/O | `KeyInput` test mode | Documented in [`console.md`](../../pascal/std/console.md) |
| Headless graph | `with_headless_graph_backend_for_tests` | Used in compiler/graph tests |

Reference tests:

- [`crates/fpas-compiler/src/tests/std_library/console/events.rs`](../../../crates/fpas-compiler/src/tests/std_library/console/events.rs) — resize, poll, timeout
- [`crates/fpas-vm/src/tests/core/tui_host_vm/`](../../../crates/fpas-vm/src/tests/core/tui_host_vm/) — TUI shortcuts, paste, lifecycle
- [`crates/fpas-compiler/src/tests/std_library/graph.rs`](../../../crates/fpas-compiler/src/tests/std_library/graph.rs) — headless graph + events

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│  fpas test my-app.fpasprj                                   │
│    discover → load project → find test entry points         │
│    optional: load *.script.toml → push events into VM       │
│    compile + run each test → collect exit code / failures   │
│    print summary (passed / failed / skipped)                │
└─────────────────────────────────────────────────────────────┘
         │                              │
         ▼                              ▼
   Std.Test (FPAS)              fpas-vm test harness
   Assert*, RunAll, Fail        queues, headless graph, output
```

### Layer 1 — `Std.Test` (language)

New standard unit. Spec draft: [`std-test.md`](std-test.md).

- Assertion procedures that set a thread-local or VM-global “test failed” flag and record message + location
- `Fail(Msg)` / `Skip(Msg)` for explicit outcomes
- Optional lightweight `TestCase` registration (Phase 2); Phase 1 can use plain `program` files that call asserts in `begin…end`

On failure the test program exits with a dedicated non-zero code (runner interprets this).

### Layer 2 — `fpas test` (CLI)

New subcommand alongside `fpas check`. Spec draft: [`runner.md`](runner.md).

Discovery rules (initial):

- Explicit path: `fpas test path/to/tests/` or `fpas test app.fpasprj`
- File naming: `*_test.fpas` under project `include` globs
- Optional project kind: `kind = "test"` with required `main` pointing at aggregator (Phase 5)

Runner behavior:

1. Parse/link like `fpas check` (same project loading in `fpas-project`)
2. For each test target: compile, create VM, apply scripted input if present, `run()`
3. Capture stdout/stderr and test failure state
4. Emit human-readable report; exit `1` if any failure

### Layer 3 — Scripted input (sidecar)

Optional TOML file beside the test or referenced from manifest. Spec draft: [`scripted-input.md`](scripted-input.md).

Example: `menu_bar_test.fpas` + `menu_bar_test.script.toml`

The runner translates TOML entries into `push_console_event` / `push_graph_event` / `push_readln_input` calls before `vm.run()`.

---

## Implementation plan

High-level phases are below. **Every task has a checkbox in [`implementation.md`](implementation.md)** — work through phases in order and tick items there.

---

## Implementation phases

### Phase 0 — Requirements

- [x] Problem statement and scope
- [x] Architecture and phased roadmap documented
- [x] Contributor task breakdown ([`implementation.md`](implementation.md))
- [ ] Review open decisions (below) and mark resolved
- [ ] Align naming with existing `Std.*` conventions

### Phase 1 — Minimal assertions + manual run

**Deliverables**

- [ ] `docs/pascal/std/test.md` — user-facing `Std.Test` spec
- [ ] `crates/fpas-std/src/test/` — runtime: assert helpers, failure flag
- [ ] `crates/fpas-sema/.../test.rs` — register `Std.Test` symbols
- [ ] `crates/fpas-compiler/.../test.rs` — lower asserts to intrinsics
- [ ] `examples/pascal/test/assert_basics.fpas` — example
- [ ] Compiler integration tests — assert pass/fail paths

**Success criteria**

- [ ] A `.fpas` program can call `Std.Test.AssertEquals(2, 1 + 1)` and exit 0
- [ ] Failed assert prints message with source location hint and exits non-zero
- [ ] `fpas examples/pascal/test/assert_basics.fpas` works (direct run before dedicated `test` subcommand)

→ Tasks: [`implementation.md` § Phase 1](implementation.md#phase-1--stdtest-assertions)

### Phase 2 — `fpas test` runner

**Deliverables**

- [ ] `crates/fpas-cli/src/cli_test.rs` — subcommand implementation
- [ ] `docs/pascal/10-projects.md` — document `fpas test`
- [ ] Discovery of `*_test.fpas` — per-project glob scan
- [ ] Summary output — `N passed, M failed`

**Success criteria**

- [ ] `fpas test examples/pascal/test/` runs all `*_test.fpas` and returns correct exit code
- [ ] Works with `.fpasprj` (includes test files + app sources for `uses`)

→ Tasks: [`implementation.md` § Phase 2](implementation.md#phase-2--fpas-test-runner)

### Phase 3 — Scripted console/TUI input

**Deliverables**

- [ ] `crates/fpas-cli/src/test_script/` — parse TOML → VM queue pushes
- [ ] `examples/pascal/test/tui_escape_test.fpas` + `.script.toml` — end-to-end TUI test
- [ ] Runner flag `--script path` — override sidecar path

**Success criteria**

- [ ] TUI program under `Application.Run` quits on simulated Escape without human input
- [ ] Mouse down/up at (x, y) reaches `OnMouse` handler in test mode

→ Tasks: [`implementation.md` § Phase 3](implementation.md#phase-3--scripted-consoletui-input)

### Phase 4 — Graph headless tests

**Deliverables**

- [ ] Runner sets headless graph backend automatically for graph tests
- [ ] Script support for `graph_key`, `graph_mouse`, `graph_wheel` events
- [ ] Optional `Std.Test.AssertPixels(...)` or Rust-only pixel hook (Phase 4b — evaluate need)

**Success criteria**

- [ ] `graph_basics`-style program runs in CI with scripted quit event
- [ ] Existing headless pixel assertions remain in Rust; optional FPAS wrapper if justified

→ Tasks: [`implementation.md` § Phase 4](implementation.md#phase-4--graph-headless-tests)

### Phase 5 — Test projects and workspace

**Deliverables**

- [ ] `kind = "test"` in `.fpasprj` — dedicated test project type
- [ ] `fpas test` with no path — discover workspace / single project like `fpas check`
- [ ] `[test]` section (optional) — `script = "…"`, `headless_graph = true` per test file

**Success criteria**

- [ ] `apps/ide/` or similar can carry a sibling `ide-tests.fpasprj`
- [ ] `fpas test` at workspace root runs all member test projects

→ Tasks: [`implementation.md` § Phase 5](implementation.md#phase-5--test-projects-and-workspace)

### Phase 6 — Quality and ergonomics

- [ ] Terminal screen snapshot API for TUI (read back buffer as string grid) — research spike
- [ ] `Setup` / `Teardown` procedures in test units
- [ ] Parallel test execution (respect VM / graph headless constraints)
- [ ] Filter: `fpas test --filter menu_bar`
- [ ] JSON report for CI (`--report json`)
- [ ] Runner timeout for hung interactive tests

→ Tasks: [`implementation.md` § Phase 6](implementation.md#phase-6--quality-and-ergonomics)

---

## Intended Rust file layout (after Phase 2)

```text
crates/fpas-std/src/test/
 ├── mod.rs           — module root, re-exports
 ├── assert.rs        — AssertEquals, AssertTrue, …
 └── state.rs         — failure flag, message buffer

crates/fpas-sema/src/std_registry/loaded/test.rs
crates/fpas-compiler/src/compiler/std_calls/test.rs

crates/fpas-cli/src/
 ├── cli_test.rs       — fpas test entry
 └── test_script/      — Phase 3
      ├── mod.rs
      ├── parse.rs     — TOML → event list
      └── apply.rs     — push into Vm

docs/pascal/std/test.md
examples/pascal/test/
 ├── assert_basics.fpas
 ├── readln_test.fpas
 └── tui_escape_test.fpas (+ .script.toml)
```

---

## Open decisions

Mark **Resolved** with `[x]` when decided; record outcome in the **Decision** column.

| Done | # | Question | Proposal |
|------|---|----------|----------|
| [ ] | 1 | Separate `kind = "test"` or only `*_test.fpas` naming? | Start with naming; add `kind = "test"` in Phase 5 |
| [ ] | 2 | Assert implementation: intrinsics vs ordinary FPAS-visible functions? | Intrinsics for reliable source locations and failure capture |
| [ ] | 3 | Exit codes: one code for assert fail vs runtime error? | Distinct codes (e.g. 1 = assert, 2 = compile, 3 = runtime) |
| [ ] | 4 | Should `Std.Test` expose captured stdout to FPAS? | Defer; runner compares output in Phase 2 via `--expect-stdout` file |
| [ ] | 5 | TUI screen assertions | Phase 6 spike; until then assert on `WriteLn` side effects or handler flags |
| [ ] | 6 | Test programs vs test units | Allow both: `program …_test` entry files and `unit MyApp.Tests` imported by aggregator |

---

## Related documentation

| Document | Relevance |
|----------|-----------|
| [`implementation.md`](implementation.md) | **Contributor guide** — checkbox task list per phase |
| [`std-test.md`](std-test.md) | Draft `Std.Test` API |
| [`runner.md`](runner.md) | `fpas test` CLI behavior |
| [`scripted-input.md`](scripted-input.md) | Sidecar event script format |
| [`docs/pascal/std/console.md`](../../pascal/std/console.md) | Event queues, test mode |
| [`docs/pascal/std/tui-app.md`](../../pascal/std/tui-app.md) | Hosted TUI under test |
| [`docs/pascal/std/graph.md`](../../pascal/std/graph.md) | Graph event model |
| [`docs/pascal/10-projects.md`](../../pascal/10-projects.md) | Project loading (extend for test) |
| [`docs/rust/project-loading.md`](../../rust/project-loading.md) | Contributor map for loader changes |
| [`tui-application-framework.md`](../tui-application-framework.md) | Phase 8 mentions headless/scripted terminal tests |

---

## Out of scope

- Replacing Rust compiler/VM unit tests — those stay for bytecode and internal invariants
- Running every file under `examples/` without classification
- OS-level input automation (AutoHotkey, SendInput) — use VM queues instead
