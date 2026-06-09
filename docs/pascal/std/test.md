# `Std.Test`

Assertion procedures for FPAS test programs. Planned runner: `fpas test` (see [`docs/future/test-framework/README.md`](../../future/test-framework/README.md)).

```pascal
program Example;
uses Std.Test;
begin
  AssertEquals(4, 2 + 2);
  AssertTrue(1 + 1 = 2);
  AssertFalse(1 = 2)
end.
```

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
| procedure | `Skip(Msg: string)` | print skip reason; does not fail |

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

Print `test skipped: …` to stderr and continue. Does not raise **F4023**.

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

## Example

See [`examples/pascal/test/assert_basics_test.fpas`](../../../examples/pascal/test/assert_basics_test.fpas).

Run:

```sh
fpas examples/pascal/test/assert_basics_test.fpas
```
