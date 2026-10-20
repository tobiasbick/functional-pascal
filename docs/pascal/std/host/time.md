# `Std.Time`

Wall-clock and monotonic time helpers plus task-aware sleep. This page is the full API for the unit.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Time as Time;

begin
  const Start: integer := Time.MonotonicMillis();
  Time.Sleep(100);
  Console.WriteLn(Time.ElapsedMillis(Start));
end.
```

`Std.Time` exposes host clock values and task-aware sleep. It is separate from `Std.Console.Delay`, which remains available for CRT-style console programs.

[`Std.Calendar`](calendar.md) provides pure Gregorian dates, UTC date-times,
ISO/RFC-3339 text conversion and checked arithmetic. Pass `TimestampMillis()`
to `Calendar.FromTimestampMillis` to obtain a calendar value. Monotonic readings
are not Unix timestamps.


## Importing and names

Import with `uses Std.Time as Time;` and access public symbols as `Time.Name`. User enum variants require `Time.Type.Variant`.

---

## Quick reference

Requires `uses Std.Time as Time;`.

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
back on the shared ready queue. The same rule applies inside synchronous hosted callbacks such as
native array `Map`: the VM retains the callback and collection-operation state as part of the owner task,
suspends that owner, and resumes the callback without repeating completed elements.

---

## `function TimestampMillis(): integer`

Returns UTC milliseconds since `1970-01-01T00:00:00Z`.

```pascal id=example-2 fragment context=body-console-time
Console.WriteLn(Time.TimestampMillis());
```

---

## `function MonotonicMillis(): integer`

Returns monotonic milliseconds since runtime initialization.

```pascal id=example-3 fragment context=body-time
const Start: integer := Time.MonotonicMillis();
```

---

## `function ElapsedMillis(Start: integer): integer`

Returns monotonic milliseconds elapsed since `Start`, a value from `MonotonicMillis`.

```pascal id=example-4 fragment context=body-console-time
const Start: integer := Time.MonotonicMillis();
Time.Sleep(50);
Console.WriteLn(Time.ElapsedMillis(Start));
```

---

## `procedure Sleep(Milliseconds: integer)`

Waits for the given number of milliseconds. Spawned tasks, including their active synchronous
callbacks, wait cooperatively without pinning a pool worker. Negative values produce a runtime
error.

```pascal id=example-5 fragment context=body-time
Time.Sleep(250);
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
- [Calendar values and text](calendar.md)
- [Standard library index](../README.md)
