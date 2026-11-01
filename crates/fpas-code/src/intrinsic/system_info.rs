//! Read-only host-information selectors.
use num_enum::TryFromPrimitive;

documented_intrinsic_enum! {
/// Queries of the per-VM host-information collector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u16)]
pub enum SystemInfoIntrinsic {
    Info = 858,
    Snapshot = 859,
    Cpu = 860,
    Memory = 861,
    Disks = 862,
    Networks = 863,
    Components = 864,
    Machine = 865,
    Gpus = 866,
    InspectProcesses = 867,
    ListUsers = 868,
    ListGroups = 869,
}
}

impl SystemInfoIntrinsic {
    /// Canonical public query name.
    pub const fn source_name(self) -> &'static str {
        use SystemInfoIntrinsic::*;
        match self {
            Info => "Std.Os.System.Info",
            Snapshot => "Std.Os.System.Snapshot",
            Cpu => "Std.Os.Hardware.Cpu",
            Memory => "Std.Os.Hardware.Memory",
            Disks => "Std.Os.Hardware.Disks",
            Networks => "Std.Os.Hardware.Networks",
            Components => "Std.Os.Hardware.Components",
            Machine => "Std.Os.Hardware.Machine",
            Gpus => "Std.Os.Hardware.Gpus",
            InspectProcesses => "Std.Os.Proc.InspectProcesses",
            ListUsers => "Std.Os.Users.ListUsers",
            ListGroups => "Std.Os.Users.ListGroups",
        }
    }
}
