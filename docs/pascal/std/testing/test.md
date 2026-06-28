# `Std.Test`

Assertion procedures for FPAS test programs. Run single tests with `fpas` or batch-discover them with `fpas test` (see [CLI](../../program-structure/cli.md) and [`docs/future/test-framework/README.md`](../../../future/test-framework/README.md)).

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
| procedure | `Skip(Msg: string)` | mark the current test skipped; runner reports `SKIP` (exit `0` unless `fpas test --strict`) |
| procedure | `PushReadLn(Line: string)` | queue one line for the next `Std.Console.ReadLn` (native tests) |
| procedure | `AssertScreenLine(Expected: string; Y: integer)` | fail when CRT row `Y` text differs (headless/TUI back buffer) |
| procedure | `AssertScreenCell(X, Y: integer; Ch: string; Fg, Bg: integer)` | fail when one CRT cell differs (`Fg`/`Bg` are packed colors `0..=15`) |
| procedure | `AssertViewRect(App: Application; V: ViewId; X, Y, W, H: integer)` | fail when view bounds differ (`uses Std.Tui` required) |

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

Mark the current test as skipped and continue. Does not raise **F4023**. The `fpas test` runner records the test as **skipped** (`SKIP` line, included in summary). Skipped tests do not fail the run unless you pass `--strict` (exit code `1` when any test was skipped).

### `procedure PushReadLn(Line: string)`

Queue one input line for the next blocking `Std.Console.ReadLn` (or line-buffered `Read`). Call before `ReadLn` in native tests instead of a `*.script.toml` readln sidecar.

### `procedure AssertScreenLine(Expected: string; Y: integer)`

Fail with **F4023** when row `Y` (one-based) of the virtual CRT back buffer does not equal `Expected`. Use after `Application.TestPump` in headless TUI tests. Requires `uses Std.Console` (or `Std.Tui`, which pulls console symbols).

### `procedure AssertScreenCell(X, Y: integer; Ch: string; Fg, Bg: integer)`

Fail with **F4023** when cell `(X, Y)` (one-based) does not match the expected character and packed CRT colors. Use `Std.Console` color constants (for example `Red`, `Black`) for `Fg` and `Bg`.

### `procedure AssertViewRect(App: Application; V: ViewId; X, Y, W, H: integer)`

Fail with **F4023** when the absolute rectangle of view handle `V` differs from `(X, Y, W, H)`. Requires `uses Std.Tui` (for `Application`, `ViewId`, and host view constructors).

---

## Setup / Teardown (test projects)

When tests run via `fpas test` inside a `kind = "test"` project, the runner may execute optional hooks from **helper units** (project sources whose basename is not `*_test.fpas`):

| Procedure | When run |
|-----------|----------|
| `Setup` | Before each test program (at most one per project) |
| `Teardown` | After each test program, even if the test failed (at most one per project) |

Hooks must be parameterless procedures. Names are matched case-insensitively. Use `uses Std.Test` in the helper unit if hooks need assertions.

See [`docs/future/test-framework/runner.md`](../../../future/test-framework/runner.md#setup--teardown-hooks-test-projects).

---

## Running tests

### Single file

```sh
fpas tests/runner/assert_basics_test.fpas
```

### Batch runner

```sh
fpas test tests/
fpas test tests/suite.fpasprj
fpas test --filter tui_escape
fpas test --report json
fpas test --strict
```

Flags and discovery rules: [CLI](../../program-structure/cli.md), [`runner.md`](../../../future/test-framework/runner.md).

---

## Scripted input (legacy)

`fpas test` still accepts optional `<test>.script.toml` sidecars for project overrides (`--script`, `[test.overrides]`). **Prefer native FPAS test APIs** for new tests:

| Need | Native API |
| ---- | ---------- |
| `ReadLn` input | `Std.Test.PushReadLn` |
| Hosted TUI input | `Application.TestSendKey`, `TestPump`, … (`docs/pascal/std/tui/app/README.md`) |
| Headless graph input | `Application.OpenForTest`, `Application.TestSendKey` |

Format and remaining script event types: [`scripted-input.md`](../../../future/test-framework/scripted-input.md).

Graph golden pixel checks (`*.expect.pixels`) still run runner-side after `Application.OpenForTest` + `Present`.

---

## Golden sidecars (runner assertions)

After a successful test run, `fpas test` may compare optional golden files beside the test. All are **runner-side** checks (not `Std.Test` procedures). Omit a sidecar to skip that check.

| Sidecar | Compares | Typical use |
|---------|----------|-------------|
| `<test>.expect.stdout` | Captured `WriteLn` lines | Console output |
| `<test>.expect.screen` | Compact CRT screen rows | Hosted `Std.Tui` paint (`GotoXY`, `ClrScr`, …) |
| `<test>.expect.pixels` | Headless graph frame spot checks (`x y 0xRRGGBB`) | `Std.Graph` after `Present` |

Details: [`runner.md`](../../../future/test-framework/runner.md) (golden stdout / screen / pixels sections).

---

## Examples

| Path | Topic |
|------|--------|
| [`runner/assert_basics_test.fpas`](../../../tests/runner/assert_basics_test.fpas) | `AssertEquals` / `AssertTrue` / `AssertFalse` |
| [`console/readln_test.fpas`](../../../tests/console/readln_test.fpas) | `PushReadLn` + `ReadLn` |
| [`console/readln_order_test.fpas`](../../../tests/console/readln_order_test.fpas) | Multiple `PushReadLn` lines in order |
| [`runner/skip_test.fpas`](../../../tests/runner/skip_test.fpas) | `Skip` + runner `SKIP` reporting |
| [`runner/stdout_echo_test.fpas`](../../../tests/runner/stdout_echo_test.fpas) | `*.expect.stdout` |
| [`tui/host/tui_escape_test.fpas`](../../../tests/tui/host/tui_escape_test.fpas) | Escape + `AssertScreenLine` (native headless API) |
| [`tui/host/tui_mouse_test.fpas`](../../../tests/tui/host/tui_mouse_test.fpas) | `TestSendMouse` + `OnMouse` (native headless API) |
| [`graph/graph_smoke_test.fpas`](../../../tests/graph/graph_smoke_test.fpas) | Headless graph (`OpenForTest` + `TestSendKey`) + `*.expect.pixels` |
| [`suite.fpasprj`](../../../tests/suite.fpasprj) | `kind = "test"` project bundle |

Manual failure demo (not auto-discovered): [`manual/assert_fail_demo.fpas`](../../../tests/manual/assert_fail_demo.fpas).

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`std_registry/loaded/test.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/test.rs) |
| Compiler | [`std_calls/test.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/test.rs) |
| Runtime | [`test/`](../../../../crates/fpas-std/src/test/) |
| Intrinsics | [`intrinsic/test.rs`](../../../../crates/fpas-bytecode/src/intrinsic/test.rs) |

## See also

- [Testing index](README.md)
- [Standard library index](../README.md)
