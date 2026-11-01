# Std.Os.System

`Std.Os.System` queries OS metadata and collects typed host-information snapshots.

Import with `uses Std.Os.System as System;`. Queries use the bundled
`sysinfo 0.40.1` library and return FPAS values. They do not launch commands or
change host settings.

## Quick reference

| Kind | Signature | Description |
| --- | --- | --- |
| function | `Info(): result of (SystemInfo, SystemError)` | Collect OS metadata |
| function | `Snapshot(): result of (HostSnapshot, SystemError)` | Explicitly collect all host-information domains |

`SystemError` has `QueryFailed(Operation: string; Reason: string)` and
`OutOfRange(Field: string)`. Backend failure text is retained. An unsigned
native counter outside the FPAS signed 64-bit range produces `OutOfRange`;
it is never wrapped or clamped.

## Metadata and partial results

`SystemInfo` has the following fields:

| Fields | Type and meaning |
| --- | --- |
| `Architecture` | `string`, architecture reported by sysinfo |
| `DistributionId` | `string`, distribution identifier; backend-defined fallback on other OSes |
| `DistributionLike` | `array of string`, related distribution identifiers |
| `Name`, `KernelVersion`, `OsVersion`, `LongOsVersion`, `HostName` | Each `result of (string, SystemError)` |
| `UptimeSeconds`, `BootTimeSeconds` | Each `result of (integer, SystemError)`; uptime duration and Unix boot timestamp |
| `OpenFilesLimit` | `result of (integer, SystemError)`; current process's open-file limit |
| `LoadAverage` | `result of (LoadAverage, SystemError)` |

`LoadAverage` contains real fields `OneMinute`, `FiveMinutes` and
`FifteenMinutes`. These are OS load averages, not percentages. A platform
without that query returns an error in the field without losing other metadata.

`HostSnapshot` contains integer Unix timestamps `StartedAtMilliseconds` and
`FinishedAtMilliseconds`. Its other fields are independent results, all using
`SystemError`:

| Field | Successful value |
| --- | --- |
| `System` | `SystemInfo` |
| `Cpu` | `Std.Os.Hardware.CpuSample` |
| `Memory` | `Std.Os.Hardware.MemoryInfo` |
| `Disks` | `Std.Os.Hardware.DiskSample` |
| `Networks` | `Std.Os.Hardware.NetworkSample` |
| `Components` | `Std.Os.Hardware.ComponentSample` |
| `Machine` | `Std.Os.Hardware.MachineInfo` |
| `Gpus` | `Std.Os.Hardware.GpuSample` |
| `Processes` | `Std.Os.Proc.ProcessSample` |
| `Users` | `array of Std.Os.Users.UserInfo` |
| `Groups` | `array of Std.Os.Users.GroupInfo` |

A section error leaves successful sections available. Collection is sequential;
the snapshot is not an atomic view of the machine. Wall-clock adjustments can
affect timestamps. Sample intervals use a separate monotonic clock.

## Sampling and lifetime

One collector belongs to each VM. Its state survives query calls and is shared
by tasks in that VM. Targeted calls and full snapshots refresh the same domain
history. Independent VMs have independent histories. The collector is created
on first use and joined at VM teardown.

Native collectors remain on their own owner thread, including Windows GPU
objects. Task queries suspend cooperatively; only one asynchronous collection
is admitted per VM and other querying tasks wait without occupying pool
workers. Native data is converted to FPAS values on the calling task.
Cancellation waits for already-submitted collection to return; sysinfo's OS
calls cannot be forcibly interrupted.

`Option` fields preserve backend absence. Fields for fallible native queries
use `Result` so their failure reason is retained. A successful empty inventory
is valid and does not prove that every device was visible. Some sysinfo
properties report empty strings or zero for unavailable data; those values
are retained because the backend supplies no more precise status.

Normal VM execution and ordinary debugging support these queries. Seeded
scheduling, recording capture and detached debugger calls reject them as
host observations.

## Example

```pascal id=host-snapshot
program HostSnapshot;
uses Std.Os.System as System, Std.Console as Console;
begin
  const Snapshot := System.Snapshot().Unwrap();
  Console.WriteLn(Snapshot.System.Unwrap().Architecture);
  case Snapshot.Memory of
    when Ok(const Memory):
      Console.WriteLn(Memory.AvailableBytes);
    when Error(const Failure):
      Console.WriteLn(Failure.Message());
  end case;
end.
```

## Platform coverage

The backend supports Windows, Linux, macOS and other targets listed in its
[documentation](https://docs.rs/sysinfo/0.40.1/sysinfo/). Individual fields and
inventories vary by OS, permissions, drivers, sandbox and hardware. This FPAS
integration is verified on Windows and Linux (WSL). macOS and other targets
require their own execution checks before claiming verification.

## Implementation (contributors)

- Shared declarations: `crates/fpas-builtins/src/host_info.rs`.
- Semantic registration: `crates/fpas-sema/src/std_registry/loaded/host_info.rs`.
- Runtime: `crates/fpas-runtime/src/execution/vm/hosted/host_info/`.
- Public field declarations: `lib/api/Std/Os.System.fpas`.

## See also

- [Hardware and resource observations](hardware.md)
- [Process inspection](proc.md#read-only-process-inspection)
- [Users and groups](users.md)
