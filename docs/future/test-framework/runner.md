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
| `--list` | Print tests that would run, do not execute |
| `--report json` | Machine-readable summary on stdout |

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

## Sidecar script discovery

If `menu_bar_test.fpas` runs, look for (first match wins):

1. `menu_bar_test.script.toml` (same directory)
2. `[test.script]` entry in `.fpasprj` keyed by source path (Phase 5)

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
