# Draft: `fpas test` runner

CLI specification draft for the test subcommand. When implemented, the canonical spec is merged into [`docs/pascal/10-projects.md`](../../pascal/10-projects.md).

---

## Command

```text
fpas test [<path>] [options]
```

| Form | Behavior |
|------|----------|
| `fpas test` | Discover like `fpas check`: workspace, then single `.fpasprj`, else error |
| `fpas test app.fpasprj` | Run tests declared by / found in project |
| `fpas test tests/` | Run all `*_test.fpas` under directory (non-project mode) |
| `fpas test one_test.fpas` | Run single test program |

---

## Options (planned)

| Flag | Description |
|------|-------------|
| `--filter <pattern>` | Substring match on test file path (case-insensitive) |
| `--fail-fast` | Stop after first failure |
| `--script <path>` | Use explicit script file instead of sidecar auto-discovery |
| `--list` | Print tests that would run to stdout, do not execute |
| `--report json` | Machine-readable summary on stdout (version 1 JSON; human summary omitted) |
| `--timeout <secs>` | Abort each test run after the given wall-clock seconds (cooperative shutdown) |
| `--jobs <n>` | Run up to `n` tests concurrently (`0` = available CPU parallelism; default `1`) |

---

## Discovery

### Phase 2 — file naming

1. Load project sources (same as `fpas check`).
2. From merged source list, select files whose basename matches `*_test.fpas` (case-insensitive).
3. Each file must contain a `program` declaration (entry point).
4. Compile and run in deterministic sorted path order.

### Phase 5 — project kind

```toml
[project]
name = "my-app-tests"
kind = "test"
main = "tests/all.fpas"

[sources]
include = ["tests/**/*.fpas", "../my-app/src/**/*.fpas"]
```

- `kind = "test"` requires `main`; cannot be executed with bare `fpas` (only `fpas test`).
- Library test projects may depend on app library via `[dependencies].projects`.

---

## Execution pipeline

For each discovered test:

```text
1. compile(test_program)     — same pipeline as fpas run
2. vm ← Vm::new(chunk)
3. if script present:
     apply_script(vm, script)   — see scripted-input.md
4. if graph test (heuristic or manifest flag):
     with_headless_graph_backend(|| vm.run())
   else:
     vm.run()
5. collect:
     - exit: Ok / AssertFail / VmError / CompileError
     - vm.output() lines (optional golden compare)
6. record TestResult { name, status, message, duration }
```

---

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | All tests passed (skipped tests do not fail run unless `--strict`) |
| `1` | One or more test failures (assert or expected output mismatch) |
| `2` | Compile / link / project load error |
| `3` | Unexpected runtime error in test harness |

---

## Output format (human)

```text
Running 3 tests...

  PASS  assert_basics_test.fpas (2ms)
  PASS  readln_test.fpas (4ms)
  FAIL  tui_escape_test.fpas (18ms)
        test assertion failed: expected quit handler to run
        at: tui_escape_test.fpas:24:3

Summary: 2 passed, 1 failed, 0 skipped (24ms)
```

---

## Golden stdout (optional)

If `greet_test.expect.stdout` exists beside `greet_test.fpas`, the runner compares captured `WriteLn` output (`vm.output().lines`) after a successful test run. Mismatch fails the test (exit code `1`) with expected/actual line hints.

- One line per `WriteLn` result; file may end with a trailing newline.
- Omitted sidecar: no stdout check (assertions-only tests unchanged).
- Setup/Teardown hook runs do not compare golden stdout.

## Golden pixels (optional, headless graph)

If `graph_smoke_test.expect.pixels` exists beside a test file, the runner spot-checks colors from the last headless `Present` frame (`last_headless_graph_frame_for_tests`) after a successful run. Requires `headless_graph = true` in the test script or manifest.

Format (one spot check per line, `#` comments allowed):

```text
# size 32 24
0 0 0x00020408
2 2 0x00FFFFFF
```

Mismatch fails the test (exit code `1`) with coordinate and color hints. Full-frame dumps are intentionally not required.

---

## Golden screen (optional, TUI)

If `menu_bar_test.expect.screen` exists beside a test file, the runner compares the compact CRT screen snapshot (`vm.screen_snapshot().compact_lines()`) after a successful run. Use this for hosted `Std.Tui` paint assertions (positioned text, not only `WriteLn` capture).

- One line per screen row; leading/trailing blank rows and trailing spaces are stripped before compare.
- `fpas-std` exports [`ScreenSnapshot`](../../../crates/fpas-std/src/console/snapshot.rs) for Rust-side inspection.
- Omitted sidecar: no screen check.

---

## Sidecar script discovery

If `menu_bar_test.fpas` runs, look for a script (first match wins):

1. `--script <path>` CLI flag
2. `[test.overrides."menu_bar_test.fpas"].script` in `.fpasprj`
3. `menu_bar_test.script.toml` beside the test file (same directory)

---

## Setup / Teardown hooks (test projects)

In `kind = "test"` projects, helper units (any `*.fpas` that is **not** `*_test.fpas`) may declare parameterless procedures named `Setup` and/or `Teardown` (case-insensitive). The runner discovers at most one of each across the project.

For every discovered test file:

```text
1. if Setup exists: compile and run synthetic hook program (linked like the test)
2. run the test program (scripts apply only to the test, not hooks)
3. if Teardown exists: run teardown even when the test failed
```

- Setup failure skips the test body; teardown still runs when declared.
- Teardown failure fails an otherwise passing test; it does not change an already-failing test outcome.
- Duplicate `Setup` or `Teardown` across multiple units is a project load error.

Example helper unit:

```pascal
unit Tests.Fixture;
uses Std.Test;
procedure Setup();
begin
  AssertTrue(true)
end;
procedure Teardown();
begin
  AssertTrue(true)
end.
```

Implementation: `crates/fpas-cli/src/cli_test/hooks.rs`, `run.rs`.

---

## Parallel execution

When `--jobs` is greater than `1` (or `0` resolves above one worker), the runner executes independent tests on worker threads. Each test still gets its own compile + VM instance; scripts and timeouts apply per test.

- Output is buffered per test and printed in discovery order after completion.
- `--fail-fast` stops scheduling new tests after the first failure; in-flight tests still finish.
- Headless graph tests are safe to run in parallel: the test backend is thread-local (`fpas-std` graph backend).
- Test programs without `go` use a single-threaded `Vm::run` fast path (no nested `thread::scope`), so worker threads do not stack scoped thread blocks.

Implementation: `crates/fpas-cli/src/cli_test/parallel.rs`, `crates/fpas-vm/src/vm/mod.rs`.

---

## Relationship to existing smoke tests

| Mechanism | Scope |
|-----------|-------|
| `cargo test -p fpas-cli non_interactive_examples_run_successfully` | Repository examples allowlist; keep |
| `fpas-compiler` Rust integration tests | Compiler/stdlib bytecode; keep |
| `fpas test` | **User and app projects**; primary author-facing path |

Non-interactive examples may later gain optional `*_test.fpas` siblings, but the allowlist remains for “runs without hanging” smoke coverage.

---

## Implementation location

- CLI entry: extend `crates/fpas-cli/src/main.rs` argument routing
- Core logic: `crates/fpas-cli/src/cli_test.rs`
- Reuse: `fpas-project` load/link, `fpas-compiler` compile, `fpas-vm` run + queues
- Script application: `crates/fpas-cli/src/test_script/`

See [`docs/rust/project-loading.md`](../../rust/project-loading.md) for loader touch points.
