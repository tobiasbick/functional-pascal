# `Std.Test`

Assertion procedures for FPAS test programs. Run single tests with `fpas` or batch-discover them with `fpas test` (see [10-projects.md](../10-projects.md) and [`docs/future/test-framework/README.md`](../../future/test-framework/README.md)).

```pascal
program Example;
uses Std.Test;
begin
  AssertEquals(4, 2 + 2);
  AssertTrue(1 + 1 = 2);
  AssertFalse(1 = 2)
end.
```

Test entry files are named `*_test.fpas` and must declare a `program` (not a bare `unit`).

**Maintenance (implementers only):** align with [`std_registry/loaded/test.rs`](../../../crates/fpas-sema/src/std_registry/loaded/test.rs), [`std_calls/test.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/test.rs), [`test/`](../../../crates/fpas-std/src/test/), and [`intrinsic/test.rs`](../../../crates/fpas-bytecode/src/intrinsic/test.rs).

---

## Importing and names

After `uses Std.Test;` use short names (`AssertEquals`, …) or qualified forms (`Std.Test.AssertEquals`, …).

---

## Quick reference

Requires `uses Std.Test;`.

| Kind | Name | Notes |
|------|------|-------|
| procedure | `AssertTrue(Cond: boolean)` | fail when `Cond` is false |
| procedure | `AssertFalse(Cond: boolean)` | fail when `Cond` is true |
| procedure | `AssertEquals(Expected; Actual)` | equality for `integer`, `boolean`, `string`, or `real` (both operands same type) |
| procedure | `Fail(Msg: string)` | unconditional failure |
| procedure | `Skip(Msg: string)` | print skip reason; runner reports `SKIP` (exit `0` unless `fpas test --strict`) |

---

## Procedures

### `procedure AssertTrue(Cond: boolean)`

Fail with diagnostic **F4023** when `Cond` is false.

### `procedure AssertFalse(Cond: boolean)`

Fail with **F4023** when `Cond` is true.

### `procedure AssertEquals(Expected; Actual)`

Fail with **F4023** when operands differ. Both arguments must have the same type: `integer`, `boolean`, `string`, or `real`. Message includes expected and actual values (strings are quoted).

### `procedure Fail(Msg: string)`

Unconditional failure with **F4023** and user message.

### `procedure Skip(Msg: string)`

Print `test skipped: …` to stderr and continue. Does not raise **F4023**. The `fpas test` runner records the test as **skipped** (`SKIP` line, included in summary). Skipped tests do not fail the run unless you pass `--strict` (exit code `1` when any test was skipped).

---

## Setup / Teardown (test projects)

When tests run via `fpas test` inside a `kind = "test"` project, the runner may execute optional hooks from **helper units** (project sources whose basename is not `*_test.fpas`):

| Procedure | When run |
|-----------|----------|
| `Setup` | Before each test program (at most one per project) |
| `Teardown` | After each test program, even if the test failed (at most one per project) |

Hooks must be parameterless procedures. Names are matched case-insensitively. Use `uses Std.Test` in the helper unit if hooks need assertions.

See [`docs/future/test-framework/runner.md`](../../future/test-framework/runner.md#setup--teardown-hooks-test-projects).

---

## Running tests

### Single file

```sh
fpas examples/pascal/test/assert_basics_test.fpas
```

### Batch runner

```sh
fpas test examples/pascal/test/
fpas test examples/pascal/test/tests.fpasprj
fpas test --filter tui_escape
fpas test --report json
fpas test --strict
```

Flags and discovery rules: [10-projects.md](../10-projects.md), [`runner.md`](../../future/test-framework/runner.md).

---

## Scripted input (interactive tests)

TUI, graph, and `ReadLn` tests pair with an optional `<test>.script.toml` sidecar that queues events before `vm.run()`. The runner auto-discovers the sidecar beside each test file; override with `fpas test --script <path>`.

Format and event types: [`scripted-input.md`](../../future/test-framework/scripted-input.md).

Example: [`tui_escape_test.fpas`](../../../examples/pascal/test/tui_escape_test.fpas) + [`tui_escape_test.script.toml`](../../../examples/pascal/test/tui_escape_test.script.toml).

Graph tests set `[config] headless_graph = true` in the script so CI never opens a native window.

---

## Golden sidecars (runner assertions)

After a successful test run, `fpas test` may compare optional golden files beside the test. All are **runner-side** checks (not `Std.Test` procedures). Omit a sidecar to skip that check.

| Sidecar | Compares | Typical use |
|---------|----------|-------------|
| `<test>.expect.stdout` | Captured `WriteLn` lines | Console output |
| `<test>.expect.screen` | Compact CRT screen rows | Hosted `Std.Tui` paint (`GotoXY`, `ClrScr`, …) |
| `<test>.expect.pixels` | Headless graph frame spot checks (`x y 0xRRGGBB`) | `Std.Graph` after `Present` |

Details: [`runner.md`](../../future/test-framework/runner.md) (golden stdout / screen / pixels sections).

---

## Examples

| Path | Topic |
|------|--------|
| [`assert_basics_test.fpas`](../../../examples/pascal/test/assert_basics_test.fpas) | `AssertEquals` / `AssertTrue` / `AssertFalse` |
| [`readln_test.fpas`](../../../examples/pascal/test/readln_test.fpas) | `ReadLn` + script sidecar |
| [`readln_order_test.fpas`](../../../examples/pascal/test/readln_order_test.fpas) | Multiple scripted `ReadLn` lines in order |
| [`skip_test.fpas`](../../../examples/pascal/test/skip_test.fpas) | `Skip` + runner `SKIP` reporting |
| [`stdout_echo_test.fpas`](../../../examples/pascal/test/stdout_echo_test.fpas) | `*.expect.stdout` |
| [`tui_escape_test.fpas`](../../../examples/pascal/test/tui_escape_test.fpas) | Hosted TUI + script + `*.expect.screen` |
| [`tui_mouse_test.fpas`](../../../examples/pascal/test/tui_mouse_test.fpas) | Mouse dispatch in hosted TUI |
| [`graph_smoke_test.fpas`](../../../examples/pascal/test/graph_smoke_test.fpas) | Headless graph + script + `*.expect.pixels` |
| [`tests.fpasprj`](../../../examples/pascal/test/tests.fpasprj) | `kind = "test"` project bundle |

Manual failure demo (not auto-discovered): [`assert_fail_demo.fpas`](../../../examples/pascal/test/assert_fail_demo.fpas).
