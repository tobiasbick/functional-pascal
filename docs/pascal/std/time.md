# `Std.Time`

Wall-clock and monotonic time helpers plus blocking sleep. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Time;
begin
  var Start: integer := MonotonicMillis();
  Sleep(100);
  WriteLn(ElapsedMillis(Start))
end.
```

`Std.Time` exposes host clock values and blocking sleep. It is separate from `Std.Console.Delay`, which remains available for CRT-style console programs.

**Maintenance (implementers only):** align with [`std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs), [`std_calls/time.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/time.rs), [`time.rs`](../../../crates/fpas-std/src/time.rs), and [`intrinsic/time.rs`](../../../crates/fpas-bytecode/src/intrinsic/time.rs).

---

## Importing and names

After `uses Std.Time;` use **`TimestampMillis`**, **`MonotonicMillis`**, **`ElapsedMillis`**, **`Sleep`**, or the fully qualified forms such as **`Std.Time.Sleep`**.

---

## Quick reference

Requires `uses Std.Time;`.

| Kind | Name | Notes |
|------|------|-------|
| function | `TimestampMillis(): integer` | UTC wall-clock milliseconds since the Unix epoch |
| function | `MonotonicMillis(): integer` | monotonic milliseconds since runtime initialization |
| function | `ElapsedMillis(Start: integer): integer` | monotonic milliseconds since `Start` |
| procedure | `Sleep(Milliseconds: integer)` | block for a non-negative millisecond count |

---

## Clock semantics

- `TimestampMillis` uses the host wall clock and can move backward or jump when the system clock is adjusted.
- `MonotonicMillis` uses a steady host timer anchored when the runtime first needs monotonic time.
- `ElapsedMillis(Start)` subtracts a previous `MonotonicMillis` reading from the current one and never returns a negative value.
- Millisecond values are integers. Host timer precision may be coarser than one millisecond on some platforms.

---

## Blocking behavior

`Sleep` blocks the thread that executes it. When called from a `go` task, only that worker thread sleeps.

---

## `function TimestampMillis(): integer`

Returns UTC milliseconds since `1970-01-01T00:00:00Z`.

```pascal
WriteLn(TimestampMillis())
```

---

## `function MonotonicMillis(): integer`

Returns monotonic milliseconds since runtime initialization.

```pascal
var Start: integer := MonotonicMillis();
```

---

## `function ElapsedMillis(Start: integer): integer`

Returns monotonic milliseconds elapsed since `Start`, a value from `MonotonicMillis`.

```pascal
var Start: integer := MonotonicMillis();
Sleep(50);
WriteLn(ElapsedMillis(Start))
```

---

## `procedure Sleep(Milliseconds: integer)`

Blocks for the given number of milliseconds. Negative values produce a runtime error.

```pascal
Sleep(250)
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`time.rs`](../../../crates/fpas-std/src/time.rs) |
| Call lowering | [`std_calls/time.rs`](../../../crates/fpas-compiler/src/compiler/std_calls/time.rs) |
| Registration | [`std_registry/loaded/time.rs`](../../../crates/fpas-sema/src/std_registry/loaded/time.rs) |
| Intrinsic ids | [`intrinsic/time.rs`](../../../crates/fpas-bytecode/src/intrinsic/time.rs) |
