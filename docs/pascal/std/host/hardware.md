# Std.Os.Hardware

`Std.Os.Hardware` queries hardware inventories and resource measurements.

Import with `uses Std.Os.Hardware as Hardware;`. Every query returns
`result of (T, Std.Os.System.SystemError)` with the successful type below.
Import `Std.Os.System` to name or match that error family explicitly.

## Quick reference

| Kind | Signature | Description |
| --- | --- | --- |
| function | `Cpu(): result of (CpuSample, Std.Os.System.SystemError)` | CPU identity, counts, frequency and usage |
| function | `Memory(): result of (MemoryInfo, Std.Os.System.SystemError)` | RAM, swap and available cgroup information |
| function | `Disks(): result of (DiskSample, Std.Os.System.SystemError)` | Mounted disk inventory and I/O counters |
| function | `Networks(): result of (NetworkSample, Std.Os.System.SystemError)` | Interface inventory and traffic counters |
| function | `Components(): result of (ComponentSample, Std.Os.System.SystemError)` | Temperature sensor inventory |
| function | `Machine(): result of (MachineInfo, Std.Os.System.SystemError)` | Product and motherboard information |
| function | `Gpus(): result of (GpuSample, Std.Os.System.SystemError)` | GPU inventory, usage and memory |

Each query refreshes only its own domain. In particular, `Memory()` does not
enumerate processes, disks, network interfaces or GPUs. Collection lifetime,
errors, concurrency and platform coverage are described in [System](system.md).

## CPU and sampling

`CpuSample` contains `SampledAtMilliseconds: integer`,
`IntervalMilliseconds: option of integer`, `PhysicalCores: result of (integer,
Std.Os.System.SystemError)`, `UsagePercent: option of real`, and
`Cpus: array of CpuInfo`.

`CpuInfo` contains strings `Name`, `Vendor`, `Brand`, integer `FrequencyMHz`,
and `UsagePercent: option of real`. Counts and CPU identities reflect sysinfo's
view of the system. They are not an effective worker budget under affinity
masks or CPU quotas. Frequencies can be zero when the OS does not report them.

The first CPU query establishes a baseline and returns no usage or interval.
A new usage reading requires at least sysinfo's minimum interval (200 ms).
Calls made earlier return `None` for usage and interval and retain the baseline.
Later values describe the interval since the last accepted CPU baseline.
Percentages are readings, not a reservation of CPU capacity.

## Memory

`MemoryInfo` contains integer fields `SampledAtMilliseconds`, `TotalBytes`,
`FreeBytes`, `AvailableBytes`, `UsedBytes`, `TotalSwapBytes`, `FreeSwapBytes` and
`UsedSwapBytes`, plus `Cgroup: option of CgroupMemory`.

`CgroupMemory` contains integer `TotalBytes`, `FreeBytes`, `FreeSwapBytes` and
`ResidentBytes`. It is the cgroup information supplied by sysinfo on Linux;
`None` does not distinguish no constraint from unavailable information.
Host RAM and swap fields are not automatically replaced by cgroup values.

Free RAM and available RAM have different meanings on some systems. The
available value accounts for reusable memory according to sysinfo's OS
implementation. Keep it separate from swap and from system commit headroom.
This API does not expose Windows commit counters or Job Object limits and
does not guarantee that a subsequent allocation will succeed.

## Disks and networks

`DiskSample` and `NetworkSample` contain `SampledAtMilliseconds: integer`,
`IntervalMilliseconds: option of integer`, and respectively
`Disks: array of DiskInfo` or `Networks: array of NetworkInfo`.
The interval is absent on first collection. Subsequent calls retain native
counter history. Newly discovered devices can have a shorter history than
the containing sample; first delta counters follow sysinfo's initialization.

| Record | Fields |
| --- | --- |
| `DiskInfo` | `Id: option of string`; strings `Name`, `Kind`, `FileSystem`, `MountPoint`; integers `TotalBytes`, `AvailableBytes`, `ReadBytes`, `WrittenBytes`, `TotalReadBytes`, `TotalWrittenBytes`; booleans `Removable`, `ReadOnly` |
| `NetworkInfo` | Strings `Name`, `MacAddress`, `State`; `IpNetworks: array of string`; integer `Mtu`; optional integers `TransmitBitsPerSecond`, `ReceiveBitsPerSecond`; integer traffic counters listed below |

Network traffic counters are `ReceivedBytes`, `TransmittedBytes`,
`TotalReceivedBytes`, `TotalTransmittedBytes`, `ReceivedPackets`,
`TransmittedPackets`, `TotalReceivedPackets`, `TotalTransmittedPackets`,
`ReceiveErrors`, `TransmitErrors`, `TotalReceiveErrors`, `TotalTransmitErrors`.
Counters without `Total` are deltas since native refresh; totals are the
cumulative values supplied by the OS. They are not rates. Disk I/O counters
use the same delta/total distinction. Backend support varies.

Disk IDs are decimal text so an unsigned native identifier remains exact.
Disk order follows mount point; network order follows interface name.
IP networks use `address/prefix` notation. Native OS strings that are not valid
Unicode use replacement characters. `State` and disk `Kind` use the backend's
text labels and must not be treated as closed FPAS enums.

## Sensors, machine and GPUs

| Record | Fields |
| --- | --- |
| `ComponentSample` | Integer `SampledAtMilliseconds`, `Components: array of ComponentInfo` |
| `ComponentInfo` | `Id: option of string`, `Label: string`; optional reals `TemperatureCelsius`, `MaximumCelsius`, `CriticalCelsius` |
| `BoardInfo` | Optional strings `Name`, `Vendor`, `Version`, `Serial`, `AssetTag` |
| `MachineInfo` | Result strings `ProductName`, `ProductFamily`, `ProductVendor`, `ProductVersion`, `ProductSerial`, `ProductSku`, `ProductUuid`; `Motherboard: result of (BoardInfo, Std.Os.System.SystemError)` |
| `GpuSample` | Integer `SampledAtMilliseconds`, optional integer `IntervalMilliseconds`, `Gpus: array of GpuInfo` |
| `GpuInfo` | `PciAddress: string`; optional strings `Vendor`, `Model`; optional real `UsagePercent`; optional integers `TotalMemoryBytes`, `UsedMemoryBytes` |

Product results use `Std.Os.System.SystemError`. A failed product or board
query leaves the other results intact. Sensor maximum is the maximum observed
by the backend; critical temperature is a hardware/driver threshold when known.

GPU support is provided by sysinfo on Windows, Linux and macOS. Usage is absent
on the first collection; platform and driver restrictions can keep it absent.
VRAM fields exclude memory shared with system RAM. In this backend release,
macOS returns no VRAM totals/usage. Apple M-series PCI addresses use sysinfo's
sentinel `ffffffff:ff:ff.255`, which is not a usable hardware identifier.

## Example

```pascal id=available-memory
program AvailableMemory;
uses Std.Os.Hardware as Hardware, Std.Console as Console;
begin
  const Memory := Hardware.Memory().Unwrap();
  Console.WriteLn('Available RAM bytes: ', Memory.AvailableBytes);
end.
```

## Implementation (contributors)

The shared schema is in `crates/fpas-builtins/src/host_info.rs`; native collection
is in `crates/fpas-runtime/src/execution/vm/hosted/host_info/`.
Complete editor declarations are in `lib/api/Std/Os.Hardware.fpas`.

## See also

- [System snapshots and collection lifetime](system.md)
- [Namespace ownership](../../../specs/std-namespaces.md)
