# Draft: `Std.Test`

User-facing specification draft for the FPAS test assertion unit. When implemented, the canonical spec moves to [`docs/pascal/std/test.md`](../../pascal/std/test.md).

---

## Unit

```pascal
uses Std.Test;
```

Registered as `Std.Test` in sema; runtime in `fpas-std/src/test/`.

---

## Design

- Assertions are **procedures** that either return normally or record failure and terminate the test run.
- On failure, print a single-line message to stderr (or stdout — align with existing runtime diagnostics) including expected/actual where applicable.
- The VM/runner sets exit code **1** on assert failure (distinct from compile/runtime errors).
- All asserts are safe to call from the main thread; task-spawned asserts are **undefined** in Phase 1 (document as main-thread only).

---

## Phase 1 API

### Core assertions

| Symbol | Signature | Behavior |
|--------|-----------|----------|
| `AssertTrue` | `procedure AssertTrue(Cond: boolean)` | Fail if `Cond` is false |
| `AssertFalse` | `procedure AssertFalse(Cond: boolean)` | Fail if `Cond` is true |
| `AssertEquals` | `procedure AssertEquals(Expected, Actual: integer)` | Fail if not equal; overloads for `boolean`, `string`, `real` in later sub-phases |
| `Fail` | `procedure Fail(Msg: string)` | Unconditional failure |
| `Skip` | `procedure Skip(Msg: string)` | Mark skipped; exit 0 with skip flag (runner reports skipped) |

### Example

```pascal
program AssertBasics;
uses Std.Test;

function Double(X: integer): integer;
begin
  return X * 2
end;

begin
  AssertEquals(4, Double(2));
  AssertTrue(Double(0) = 0);
  AssertFalse(Double(1) = 3)
end.
```

---

## Phase 2 API (optional registration)

Lightweight test case list for one file aggregating multiple procedures:

| Symbol | Signature | Behavior |
|--------|-----------|----------|
| `Run` | `procedure Run(Name: string; Body: procedure)` | Register and immediately run `Body`; catch failures under `Name` |
| `RunAll` | `procedure RunAll()` | Run all registered cases (if registration model is retained) |

Alternative (simpler): **no registration** — one `program` per test file, multiple files discovered by `fpas test`. Prefer simplicity unless users request grouping.

---

## Phase 4+ (deferred)

| Symbol | Notes |
|--------|-------|
| `AssertRaises` | Expect runtime error / panic; needs VM cooperation |
| `AssertOutputContains` | Compare captured stdout substring |
| `AssertApproxEquals` | `real` with epsilon |

---

## Diagnostics

Failed assert message shape (LLM-friendly, consistent with compiler style):

```text
test assertion failed: expected 4, got 5
  hint: check the expression passed to AssertEquals
  at: assert_basics_test.fpas:12:3
```

Source location comes from intrinsic lowering (same mechanism as runtime errors elsewhere).

---

## Interaction with `fpas test`

- Direct `fpas my_test.fpas`: assert failure → process exit 1
- `fpas test`: runner counts failures, continues or stops based on `--fail-fast` (default: run all)

---

## Not in `Std.Test`

- Event scripting (runner sidecar — see [`scripted-input.md`](scripted-input.md))
- Project discovery (CLI — see [`runner.md`](runner.md))
- Headless graph mode toggle (runner/env, not user API in Phase 1)
