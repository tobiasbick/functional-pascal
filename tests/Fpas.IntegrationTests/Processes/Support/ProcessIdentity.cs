using Microsoft.Win32.SafeHandles;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.Runtime.InteropServices;

namespace Fpas.IntegrationTests.Processes.Support;

internal static class ProcessIdentity
{
    internal static int Parent(int pid)
    {
        if (OperatingSystem.IsLinux()) return int.Parse(LinuxFields(pid)[1], CultureInfo.InvariantCulture);
        using var snapshot = CreateToolhelp32Snapshot(2, 0);
        if (snapshot.IsInvalid) throw new Win32Exception(Marshal.GetLastPInvokeError());
        var entry = new ProcessEntry { Size = (uint)Marshal.SizeOf<ProcessEntry>(), Executable = "" };
        if (Process32First(snapshot, ref entry))
            do { if (entry.ProcessId == pid) return (int)entry.ParentProcessId; } while (Process32Next(snapshot, ref entry));
        throw new InvalidOperationException("Target process is absent from the process snapshot.");
    }

    internal static void AssertOwned(Process target, int adapter)
    {
        if (OperatingSystem.IsLinux()) Assert.Equal(adapter, int.Parse(LinuxFields(target.Id)[2], CultureInfo.InvariantCulture));
        else
        {
            if (!IsProcessInJob(target.Handle, IntPtr.Zero, out var inJob)) throw new Win32Exception(Marshal.GetLastPInvokeError());
            Assert.True(inJob);
        }
    }

    private static string[] LinuxFields(int pid)
    {
        var stat = File.ReadAllText("/proc/" + pid + "/stat");
        return stat[(stat.LastIndexOf(')') + 2)..].Split(' ');
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct ProcessEntry
    {
        internal uint Size, Usage, ProcessId;
        internal nuint DefaultHeapId;
        internal uint ModuleId, Threads, ParentProcessId;
        internal int Priority;
        internal uint Flags;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 260)] internal string Executable;
    }

    [DllImport("kernel32.dll", SetLastError = true)] private static extern SafeFileHandle CreateToolhelp32Snapshot(uint flags, uint pid);
    [DllImport("kernel32.dll", EntryPoint = "Process32FirstW", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)] private static extern bool Process32First(SafeFileHandle snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", EntryPoint = "Process32NextW", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)] private static extern bool Process32Next(SafeFileHandle snapshot, ref ProcessEntry entry);
    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)] private static extern bool IsProcessInJob(IntPtr process, IntPtr job, [MarshalAs(UnmanagedType.Bool)] out bool inJob);
}
