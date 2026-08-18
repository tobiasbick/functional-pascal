# `Std.Time`

Wall-clock and monotonic time helpers plus task-aware sleep. This page is the full API for the unit.

```pascal
program Example;
uses Std.Console, Std.Time;
begin
  var Start: integer := MonotonicMillis();
  Sleep(100);
  WriteLn(ElapsedMillis(Start))
end.
```

`Std.Time` exposes host clock values and task-aware sleep. It is separate from `Std.Console.Delay`, which remains available for CRT-style console programs.


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
| procedure | `Sleep(Milliseconds: integer)` | wait for a non-negative millisecond count |

---

## Clock semantics

- `TimestampMillis` uses the host wall clock and can move backward or jump when the system clock is adjusted.
- `MonotonicMillis` uses a steady host timer anchored when the runtime first needs monotonic time.
- `ElapsedMillis(Start)` subtracts a previous `MonotonicMillis` reading from the current one and never returns a negative value.
- Millisecond values are integers. Host timer precision may be coarser than one millisecond on some platforms.

---

## Scheduling behavior

`Sleep` blocks the main program thread when called by the main task. When called from a spawned
`go` task, the VM suspends that task in its cooperative timer queue and immediately releases the
pool worker to run other ready tasks. After the deadline, the timer driver places the suspended task
back on the shared ready queue.

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

Waits for the given number of milliseconds. Spawned tasks wait cooperatively without pinning a pool
worker. Negative values produce a runtime error.

```pascal
Sleep(250)
```

---

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Runtime execution | [`time.rs`](../../../../crates/fpas-std/src/time.rs) |
| Spawned-task timers | [`tasks/mod.rs`](../../../../crates/fpas-vm/src/vm/tasks/mod.rs), [`timers.rs`](../../../../crates/fpas-vm/src/vm/shared/timers.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Registration | [`std_registry/loaded/time.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/time.rs) |
| Intrinsic ids | [`intrinsic/time.rs`](../../../../crates/fpas-bytecode/src/intrinsic/time.rs) |

## See also

- [Host I/O index](README.md)
- [Standard library index](../README.md)
