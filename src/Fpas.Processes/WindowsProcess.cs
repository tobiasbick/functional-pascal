using Microsoft.Win32.SafeHandles;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Processes;

internal static class WindowsProcess
{
    private static readonly object launchGate = new();
    internal sealed record Backend(Process Process, Stream Input, Stream Output, Stream Error, SafeFileHandle Job);

    internal static Backend Start(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment, ProcessStdio stdio)
    {
        // Child pipe endpoints must be closed before another owned launch can inherit handles.
        lock (launchGate) return StartCore(path, arguments, cwd, environment, stdio);
    }

    private static Backend StartCore(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment, ProcessStdio stdio)
    {
        var security = new SecurityAttributes { Length = Marshal.SizeOf<SecurityAttributes>(), Inherit = true };
        SafeFileHandle? inputRead = null, inputWrite = null, outputRead = null, outputWrite = null, errorRead = null, errorWrite = null;
        var job = CreateJobObject(IntPtr.Zero, null);
        ProcessInfo process = default;
        SafeFileHandle? processHandle = null, thread = null;
        Process? tracked = null;
        var environmentBlock = EnvironmentBlock(environment);
        try
        {
            Check(!job.IsInvalid);
            var limits = new ExtendedLimits(); limits.Basic.Flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            Check(SetInformationJobObject(job, 9, ref limits, Marshal.SizeOf<ExtendedLimits>()));
            if (stdio == ProcessStdio.Capture)
            {
                Check(CreatePipe(out inputRead, out inputWrite, ref security, 0));
                Check(SetHandleInformation(inputWrite, 1, 0));
            }
            else inputRead = Inherit(-10);
            if (stdio != ProcessStdio.Inherit)
            {
                Check(CreatePipe(out outputRead, out outputWrite, ref security, 0));
                Check(CreatePipe(out errorRead, out errorWrite, ref security, 0));
                Check(SetHandleInformation(outputRead, 1, 0));
                Check(SetHandleInformation(errorRead, 1, 0));
            }
            else { outputWrite = Inherit(-11); errorWrite = Inherit(-12); }
            var startup = new StartupInfo
            {
                Size = Marshal.SizeOf<StartupInfo>(), Flags = 0x100,
                Input = inputRead.DangerousGetHandle(), Output = outputWrite.DangerousGetHandle(), Error = errorWrite.DangerousGetHandle()
            };
            var commandLine = Quote(path) + " " + string.Join(' ', arguments.Select(Quote));
            Check(CreateProcess(path, new StringBuilder(commandLine), IntPtr.Zero, IntPtr.Zero,
                true, stdio == ProcessStdio.Capture ? 0x08000404u : 0x00000404u, environmentBlock, cwd, ref startup, out process)); // suspended, Unicode environment
            processHandle = new SafeFileHandle(process.Process, ownsHandle: true);
            thread = new SafeFileHandle(process.Thread, ownsHandle: true);
            Check(AssignProcessToJobObject(job, processHandle));
            tracked = Process.GetProcessById((int)process.Id);
            _ = tracked.SafeHandle;
            Check(ResumeThread(thread) != uint.MaxValue);
            inputRead.Dispose(); outputWrite.Dispose(); errorWrite.Dispose();
            return new(tracked,
                inputWrite is null ? Stream.Null : new FileStream(inputWrite, FileAccess.Write),
                outputRead is null ? Stream.Null : new FileStream(outputRead, FileAccess.Read), errorRead is null ? Stream.Null : new FileStream(errorRead, FileAccess.Read), job);
        }
        catch
        {
            tracked?.Dispose();
            if (process.Process != IntPtr.Zero) TerminateProcess(process.Process, 1);
            inputRead?.Dispose(); inputWrite?.Dispose(); outputRead?.Dispose(); outputWrite?.Dispose(); errorRead?.Dispose(); errorWrite?.Dispose();
            job.Dispose(); throw;
        }
        finally { thread?.Dispose(); processHandle?.Dispose(); if (environmentBlock != IntPtr.Zero) Marshal.FreeHGlobal(environmentBlock); }
    }

    private static SafeFileHandle Inherit(int standard)
    {
        var handle = GetStdHandle(standard);
        if (handle == IntPtr.Zero || handle == new IntPtr(-1)) return new(IntPtr.Zero, ownsHandle: false);
        Check(DuplicateHandle(GetCurrentProcess(), handle, GetCurrentProcess(), out var copy, 0, true, 2));
        return copy;
    }

    private static IntPtr EnvironmentBlock(IReadOnlyDictionary<string, string>? overrides)
    {
        if (overrides is null) return IntPtr.Zero;
        var variables = new SortedDictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        foreach (System.Collections.DictionaryEntry item in Environment.GetEnvironmentVariables()) variables[(string)item.Key] = (string)item.Value!;
        foreach (var (key, value) in overrides) variables[key] = value;
        return Marshal.StringToHGlobalUni(string.Concat(variables.Select(pair => pair.Key + "=" + pair.Value + '\0')) + '\0');
    }

    internal static async Task TerminateAndWait(SafeFileHandle job, CancellationToken token)
    {
        Check(TerminateJobObject(job, 1));
        var accounting = new BasicAccounting();
        while (true)
        {
            Check(QueryInformationJobObject(job, 1, ref accounting, Marshal.SizeOf<BasicAccounting>(), IntPtr.Zero));
            if (accounting.ActiveProcesses == 0) return;
            await Task.Delay(10, token);
        }
    }

    private static string Quote(string value)
    {
        var result = new StringBuilder("\""); var slashes = 0;
        foreach (var c in value)
        {
            if (c == '\\') { slashes++; continue; }
            result.Append('\\', c == '"' ? slashes * 2 + 1 : slashes).Append(c); slashes = 0;
        }
        return result.Append('\\', slashes * 2).Append('"').ToString();
    }

    private static void Check(bool success)
    {
        if (!success) throw new Win32Exception(Marshal.GetLastPInvokeError());
    }

    [StructLayout(LayoutKind.Sequential)] private struct SecurityAttributes { internal int Length; internal IntPtr Descriptor; [MarshalAs(UnmanagedType.Bool)] internal bool Inherit; }
    [StructLayout(LayoutKind.Sequential)] private struct BasicLimits
    {
        internal long ProcessTime, JobTime; internal uint Flags; internal nuint MinWorking, MaxWorking;
        internal uint ActiveProcesses; internal nuint Affinity; internal uint Priority, Scheduling;
    }
    [StructLayout(LayoutKind.Sequential)] private struct ExtendedLimits
    {
        internal BasicLimits Basic; internal ulong ReadOps, WriteOps, OtherOps, ReadBytes, WriteBytes, OtherBytes;
        internal nuint ProcessMemory, JobMemory, PeakProcessMemory, PeakJobMemory;
    }
    [StructLayout(LayoutKind.Sequential)] private struct BasicAccounting
    {
        internal long UserTime, KernelTime, PeriodUserTime, PeriodKernelTime;
        internal uint PageFaultCount, TotalProcesses, ActiveProcesses, TotalTerminatedProcesses;
    }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] private struct StartupInfo
    {
        internal int Size; internal string? Reserved, Desktop, Title;
        internal uint X, Y, Width, Height, Columns, Rows, Fill, Flags;
        internal ushort Show, ReservedSize; internal IntPtr ReservedData, Input, Output, Error;
    }
    [StructLayout(LayoutKind.Sequential)] private struct ProcessInfo { internal IntPtr Process, Thread; internal uint Id, ThreadId; }
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern SafeFileHandle CreateJobObject(IntPtr attributes, string? name);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool SetInformationJobObject(SafeFileHandle job, int kind, ref ExtendedLimits limits, int size);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool AssignProcessToJobObject(SafeFileHandle job, SafeFileHandle process);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool TerminateJobObject(SafeFileHandle job, uint exitCode);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool QueryInformationJobObject(SafeFileHandle job, int type, ref BasicAccounting value, int length, IntPtr returned);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool CreatePipe(out SafeFileHandle read, out SafeFileHandle write, ref SecurityAttributes attributes, int size);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool SetHandleInformation(SafeFileHandle handle, uint mask, uint flags);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool CreateProcess(string application, StringBuilder command, IntPtr processAttributes, IntPtr threadAttributes, bool inherit, uint flags, IntPtr environment, string cwd, ref StartupInfo startup, out ProcessInfo process);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint ResumeThread(SafeFileHandle thread);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool TerminateProcess(IntPtr process, uint code);
    [DllImport("kernel32.dll")] private static extern IntPtr GetCurrentProcess();
    [DllImport("kernel32.dll", SetLastError = true)] private static extern IntPtr GetStdHandle(int standard);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool DuplicateHandle(IntPtr sourceProcess, IntPtr source, IntPtr targetProcess, out SafeFileHandle target, uint access, bool inherit, uint options);
}
