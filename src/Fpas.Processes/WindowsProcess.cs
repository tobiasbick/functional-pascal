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

    internal static Backend Start(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment)
    {
        // Child pipe endpoints must be closed before another owned launch can inherit handles.
        lock (launchGate) return StartCore(path, arguments, cwd, environment);
    }

    private static Backend StartCore(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment)
    {
        var security = new SecurityAttributes { Length = Marshal.SizeOf<SecurityAttributes>(), Inherit = true };
        SafeFileHandle? inputRead = null, inputWrite = null, outputRead = null, outputWrite = null, errorRead = null, errorWrite = null;
        var job = CreateJobObject(IntPtr.Zero, null);
        ProcessInfo process = default;
        SafeFileHandle? processHandle = null, thread = null;
        var environmentBlock = EnvironmentBlock(environment);
        try
        {
            Check(!job.IsInvalid);
            var limits = new ExtendedLimits(); limits.Basic.Flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            Check(SetInformationJobObject(job, 9, ref limits, Marshal.SizeOf<ExtendedLimits>()));
            Check(CreatePipe(out inputRead, out inputWrite, ref security, 0));
            Check(CreatePipe(out outputRead, out outputWrite, ref security, 0));
            Check(CreatePipe(out errorRead, out errorWrite, ref security, 0));
            Check(SetHandleInformation(inputWrite, 1, 0));
            Check(SetHandleInformation(outputRead, 1, 0));
            Check(SetHandleInformation(errorRead, 1, 0));
            var startup = new StartupInfo
            {
                Size = Marshal.SizeOf<StartupInfo>(), Flags = 0x100,
                Input = inputRead.DangerousGetHandle(), Output = outputWrite.DangerousGetHandle(), Error = errorWrite.DangerousGetHandle()
            };
            var commandLine = Quote(path) + " " + string.Join(' ', arguments.Select(Quote));
            Check(CreateProcess(path, new StringBuilder(commandLine), IntPtr.Zero, IntPtr.Zero,
                true, 0x08000404, environmentBlock, cwd, ref startup, out process)); // suspended, Unicode environment, no console
            processHandle = new SafeFileHandle(process.Process, ownsHandle: true);
            thread = new SafeFileHandle(process.Thread, ownsHandle: true);
            Check(AssignProcessToJobObject(job, processHandle));
            Check(ResumeThread(thread) != uint.MaxValue);
            inputRead.Dispose(); outputWrite.Dispose(); errorWrite.Dispose();
            return new(Process.GetProcessById((int)process.Id),
                new FileStream(inputWrite, FileAccess.Write), new FileStream(outputRead, FileAccess.Read), new FileStream(errorRead, FileAccess.Read), job);
        }
        catch
        {
            if (process.Process != IntPtr.Zero) TerminateProcess(process.Process, 1);
            inputRead?.Dispose(); inputWrite?.Dispose(); outputRead?.Dispose(); outputWrite?.Dispose(); errorRead?.Dispose(); errorWrite?.Dispose();
            job.Dispose(); throw;
        }
        finally { thread?.Dispose(); processHandle?.Dispose(); if (environmentBlock != IntPtr.Zero) Marshal.FreeHGlobal(environmentBlock); }
    }

    private static IntPtr EnvironmentBlock(IReadOnlyDictionary<string, string>? overrides)
    {
        if (overrides is null) return IntPtr.Zero;
        var variables = new SortedDictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        foreach (System.Collections.DictionaryEntry item in Environment.GetEnvironmentVariables()) variables[(string)item.Key] = (string)item.Value!;
        foreach (var (key, value) in overrides) variables[key] = value;
        return Marshal.StringToHGlobalUni(string.Concat(variables.Select(pair => pair.Key + "=" + pair.Value + '\0')) + '\0');
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
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool CreatePipe(out SafeFileHandle read, out SafeFileHandle write, ref SecurityAttributes attributes, int size);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool SetHandleInformation(SafeFileHandle handle, uint mask, uint flags);
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool CreateProcess(string application, StringBuilder command, IntPtr processAttributes, IntPtr threadAttributes, bool inherit, uint flags, IntPtr environment, string cwd, ref StartupInfo startup, out ProcessInfo process);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern uint ResumeThread(SafeFileHandle thread);
    [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool TerminateProcess(IntPtr process, uint code);
}
