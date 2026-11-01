//! Typed host-information declarations shared by the compiler and runtime.

/// A detached host-information value shape.
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// Boolean flag.
    Boolean,
    /// Checked signed 64-bit integer.
    Integer,
    /// Floating point reading.
    Real,
    /// Unicode text.
    String,
    /// Missing or available reading.
    Option(&'static Shape),
    /// Ordered collection.
    Array(&'static Shape),
    /// Nominal record.
    Record(&'static str),
    /// Independently fallible section of a full collection.
    Result(&'static Shape),
}

/// One public record and its ordered fields.
pub struct RecordDeclaration {
    /// Canonical type name.
    pub name: &'static str,
    /// Fields in declaration order.
    pub fields: &'static [(&'static str, Shape)],
}

use Shape::*;

/// All host-information record declarations.
pub const RECORDS: &[RecordDeclaration] = &[
    RecordDeclaration {
        name: "Std.Os.System.SystemInfo",
        fields: &[
            ("Name", Result(&String)),
            ("KernelVersion", Result(&String)),
            ("OsVersion", Result(&String)),
            ("LongOsVersion", Result(&String)),
            ("HostName", Result(&String)),
            ("Architecture", String),
            ("DistributionId", String),
            ("DistributionLike", Array(&String)),
            ("UptimeSeconds", Result(&Integer)),
            ("BootTimeSeconds", Result(&Integer)),
            ("LoadAverage", Result(&Record("Std.Os.System.LoadAverage"))),
            ("OpenFilesLimit", Result(&Integer)),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.System.LoadAverage",
        fields: &[
            ("OneMinute", Real),
            ("FiveMinutes", Real),
            ("FifteenMinutes", Real),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.CpuInfo",
        fields: &[
            ("Name", String),
            ("Vendor", String),
            ("Brand", String),
            ("FrequencyMHz", Integer),
            ("UsagePercent", Option(&Real)),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.CpuSample",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            ("IntervalMilliseconds", Option(&Integer)),
            ("PhysicalCores", Result(&Integer)),
            ("UsagePercent", Option(&Real)),
            ("Cpus", Array(&Record("Std.Os.Hardware.CpuInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.CgroupMemory",
        fields: &[
            ("TotalBytes", Integer),
            ("FreeBytes", Integer),
            ("FreeSwapBytes", Integer),
            ("ResidentBytes", Integer),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.MemoryInfo",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            ("TotalBytes", Integer),
            ("FreeBytes", Integer),
            ("AvailableBytes", Integer),
            ("UsedBytes", Integer),
            ("TotalSwapBytes", Integer),
            ("FreeSwapBytes", Integer),
            ("UsedSwapBytes", Integer),
            ("Cgroup", Option(&Record("Std.Os.Hardware.CgroupMemory"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.DiskInfo",
        fields: &[
            ("Id", Option(&String)),
            ("Name", String),
            ("Kind", String),
            ("FileSystem", String),
            ("MountPoint", String),
            ("TotalBytes", Integer),
            ("AvailableBytes", Integer),
            ("Removable", Boolean),
            ("ReadOnly", Boolean),
            ("ReadBytes", Integer),
            ("WrittenBytes", Integer),
            ("TotalReadBytes", Integer),
            ("TotalWrittenBytes", Integer),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.DiskSample",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            ("IntervalMilliseconds", Option(&Integer)),
            ("Disks", Array(&Record("Std.Os.Hardware.DiskInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.NetworkInfo",
        fields: &[
            ("Name", String),
            ("MacAddress", String),
            ("IpNetworks", Array(&String)),
            ("Mtu", Integer),
            ("State", String),
            ("TransmitBitsPerSecond", Option(&Integer)),
            ("ReceiveBitsPerSecond", Option(&Integer)),
            ("ReceivedBytes", Integer),
            ("TransmittedBytes", Integer),
            ("TotalReceivedBytes", Integer),
            ("TotalTransmittedBytes", Integer),
            ("ReceivedPackets", Integer),
            ("TransmittedPackets", Integer),
            ("TotalReceivedPackets", Integer),
            ("TotalTransmittedPackets", Integer),
            ("ReceiveErrors", Integer),
            ("TransmitErrors", Integer),
            ("TotalReceiveErrors", Integer),
            ("TotalTransmitErrors", Integer),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.NetworkSample",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            ("IntervalMilliseconds", Option(&Integer)),
            ("Networks", Array(&Record("Std.Os.Hardware.NetworkInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.ComponentSample",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            (
                "Components",
                Array(&Record("Std.Os.Hardware.ComponentInfo")),
            ),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.ComponentInfo",
        fields: &[
            ("Id", Option(&String)),
            ("Label", String),
            ("TemperatureCelsius", Option(&Real)),
            ("MaximumCelsius", Option(&Real)),
            ("CriticalCelsius", Option(&Real)),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.MachineInfo",
        fields: &[
            ("ProductName", Result(&String)),
            ("ProductFamily", Result(&String)),
            ("ProductVendor", Result(&String)),
            ("ProductVersion", Result(&String)),
            ("ProductSerial", Result(&String)),
            ("ProductSku", Result(&String)),
            ("ProductUuid", Result(&String)),
            ("Motherboard", Result(&Record("Std.Os.Hardware.BoardInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.BoardInfo",
        fields: &[
            ("Name", Option(&String)),
            ("Vendor", Option(&String)),
            ("Version", Option(&String)),
            ("Serial", Option(&String)),
            ("AssetTag", Option(&String)),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.GpuInfo",
        fields: &[
            ("PciAddress", String),
            ("Vendor", Option(&String)),
            ("Model", Option(&String)),
            ("UsagePercent", Option(&Real)),
            ("TotalMemoryBytes", Option(&Integer)),
            ("UsedMemoryBytes", Option(&Integer)),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Hardware.GpuSample",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            ("IntervalMilliseconds", Option(&Integer)),
            ("Gpus", Array(&Record("Std.Os.Hardware.GpuInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Proc.ProcessInfo",
        fields: &[
            ("Pid", Integer),
            ("ParentPid", Option(&Integer)),
            ("Name", String),
            ("Status", String),
            ("Executable", Option(&String)),
            ("WorkingDirectory", Option(&String)),
            ("RootDirectory", Option(&String)),
            ("Arguments", Array(&String)),
            ("Environment", Array(&String)),
            ("MemoryBytes", Integer),
            ("VirtualMemoryBytes", Integer),
            ("StartTimeSeconds", Integer),
            ("RunTimeSeconds", Integer),
            ("CpuUsagePercent", Option(&Real)),
            ("CpuTimeMilliseconds", Integer),
            ("ReadBytes", Integer),
            ("WrittenBytes", Integer),
            ("TotalReadBytes", Integer),
            ("TotalWrittenBytes", Integer),
            ("UserId", Option(&String)),
            ("EffectiveUserId", Option(&String)),
            ("GroupId", Option(&String)),
            ("EffectiveGroupId", Option(&String)),
            ("SessionId", Option(&Integer)),
            ("ThreadKind", Option(&String)),
            ("Tasks", Option(&Array(&Integer))),
            ("OpenFiles", Option(&Integer)),
            ("OpenFilesLimit", Option(&Integer)),
            ("GpuUsagePercent", Option(&Real)),
            ("GpuMemoryBytes", Option(&Integer)),
            ("Cgroup", Option(&Record("Std.Os.Hardware.CgroupMemory"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Proc.ProcessSample",
        fields: &[
            ("SampledAtMilliseconds", Integer),
            ("IntervalMilliseconds", Option(&Integer)),
            ("Processes", Array(&Record("Std.Os.Proc.ProcessInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.Users.GroupInfo",
        fields: &[("Id", String), ("Name", String)],
    },
    RecordDeclaration {
        name: "Std.Os.Users.UserInfo",
        fields: &[
            ("Id", String),
            ("GroupId", String),
            ("Name", String),
            ("Groups", Array(&Record("Std.Os.Users.GroupInfo"))),
        ],
    },
    RecordDeclaration {
        name: "Std.Os.System.HostSnapshot",
        fields: &[
            ("StartedAtMilliseconds", Integer),
            ("FinishedAtMilliseconds", Integer),
            ("System", Result(&Record("Std.Os.System.SystemInfo"))),
            ("Cpu", Result(&Record("Std.Os.Hardware.CpuSample"))),
            ("Memory", Result(&Record("Std.Os.Hardware.MemoryInfo"))),
            ("Disks", Result(&Record("Std.Os.Hardware.DiskSample"))),
            ("Networks", Result(&Record("Std.Os.Hardware.NetworkSample"))),
            (
                "Components",
                Result(&Record("Std.Os.Hardware.ComponentSample")),
            ),
            ("Machine", Result(&Record("Std.Os.Hardware.MachineInfo"))),
            ("Gpus", Result(&Record("Std.Os.Hardware.GpuSample"))),
            ("Processes", Result(&Record("Std.Os.Proc.ProcessSample"))),
            ("Users", Result(&Array(&Record("Std.Os.Users.UserInfo")))),
            ("Groups", Result(&Array(&Record("Std.Os.Users.GroupInfo")))),
        ],
    },
];

/// Targeted queries and their successful result shape.
pub const QUERIES: &[(&str, Shape)] = &[
    ("Std.Os.System.Info", Record("Std.Os.System.SystemInfo")),
    (
        "Std.Os.System.Snapshot",
        Record("Std.Os.System.HostSnapshot"),
    ),
    ("Std.Os.Hardware.Cpu", Record("Std.Os.Hardware.CpuSample")),
    (
        "Std.Os.Hardware.Memory",
        Record("Std.Os.Hardware.MemoryInfo"),
    ),
    (
        "Std.Os.Hardware.Disks",
        Record("Std.Os.Hardware.DiskSample"),
    ),
    (
        "Std.Os.Hardware.Networks",
        Record("Std.Os.Hardware.NetworkSample"),
    ),
    (
        "Std.Os.Hardware.Components",
        Record("Std.Os.Hardware.ComponentSample"),
    ),
    (
        "Std.Os.Hardware.Machine",
        Record("Std.Os.Hardware.MachineInfo"),
    ),
    ("Std.Os.Hardware.Gpus", Record("Std.Os.Hardware.GpuSample")),
    (
        "Std.Os.Proc.InspectProcesses",
        Record("Std.Os.Proc.ProcessSample"),
    ),
    (
        "Std.Os.Users.ListUsers",
        Array(&Record("Std.Os.Users.UserInfo")),
    ),
    (
        "Std.Os.Users.ListGroups",
        Array(&Record("Std.Os.Users.GroupInfo")),
    ),
];

/// Look up a detached host record.
pub fn record(name: &str) -> std::option::Option<&'static RecordDeclaration> {
    RECORDS.iter().find(|r| r.name.eq_ignore_ascii_case(name))
}

/// Public hardware declarations.
pub const HARDWARE_SYMBOLS: &[&str] = &[
    "Std.Os.Hardware.Cpu",
    "Std.Os.Hardware.Memory",
    "Std.Os.Hardware.Disks",
    "Std.Os.Hardware.Networks",
    "Std.Os.Hardware.Components",
    "Std.Os.Hardware.Machine",
    "Std.Os.Hardware.Gpus",
    "Std.Os.Hardware.CpuInfo",
    "Std.Os.Hardware.CpuSample",
    "Std.Os.Hardware.CgroupMemory",
    "Std.Os.Hardware.MemoryInfo",
    "Std.Os.Hardware.DiskInfo",
    "Std.Os.Hardware.DiskSample",
    "Std.Os.Hardware.NetworkInfo",
    "Std.Os.Hardware.NetworkSample",
    "Std.Os.Hardware.ComponentInfo",
    "Std.Os.Hardware.ComponentSample",
    "Std.Os.Hardware.MachineInfo",
    "Std.Os.Hardware.GpuInfo",
    "Std.Os.Hardware.GpuSample",
    "Std.Os.Hardware.BoardInfo",
];
/// Public system declarations.
pub const SYSTEM_SYMBOLS: &[&str] = &[
    "Std.Os.System.Info",
    "Std.Os.System.Snapshot",
    "Std.Os.System.SystemInfo",
    "Std.Os.System.LoadAverage",
    "Std.Os.System.HostSnapshot",
    "Std.Os.System.SystemError",
];
/// Public account declarations.
pub const USERS_SYMBOLS: &[&str] = &[
    "Std.Os.Users.ListUsers",
    "Std.Os.Users.ListGroups",
    "Std.Os.Users.UserInfo",
    "Std.Os.Users.GroupInfo",
];
