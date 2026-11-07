using Microsoft.Win32.SafeHandles;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.Processes.Terminal;

internal sealed partial class WindowsTerminal : PseudoTerminal
{
    private readonly Process process;
    private readonly SafeFileHandle job;
    private readonly FileStream input, output;
    private IntPtr console;
    private Task? termination;
    public override Stream Input => input;
    public override Stream Output => output;
    public override int Id => process.Id;
    public override Task<int> Completion { get; }

    private WindowsTerminal(Process process, SafeFileHandle job, IntPtr console, SafeFileHandle input, SafeFileHandle output)
    {
        this.process = process; this.job = job; this.console = console;
        this.input = new(input, FileAccess.Write); this.output = new(output, FileAccess.Read);
        Completion = Complete();
    }
    private async Task<int> Complete() { await process.WaitForExitAsync(); return process.ExitCode; }

    internal new static PseudoTerminal Start(string path, IReadOnlyList<string> arguments, string cwd, int columns, int rows)
    {
        SafeFileHandle? read = null, write = null, outputRead = null, outputWrite = null, job = null, process = null, thread = null;
        var console = IntPtr.Zero; var initialized = false;
        var startup = new StartupInfoEx(); startup.Startup.Size = Marshal.SizeOf<StartupInfoEx>(); startup.Startup.Flags = 0x100;
        try
        {
            Check(CreatePipe(out read, out write, IntPtr.Zero, 0)); Check(CreatePipe(out outputRead, out outputWrite, IntPtr.Zero, 0));
            Marshal.ThrowExceptionForHR(CreatePseudoConsole(new() { X = (short)columns, Y = (short)rows }, read, outputWrite, 0, out console));
            job = CreateJobObject(IntPtr.Zero, null); Check(!job.IsInvalid);
            var limits = new ExtendedLimits(); limits.Basic.Flags = 0x2000;
            Check(SetInformationJobObject(job, 9, ref limits, Marshal.SizeOf<ExtendedLimits>()));
            nuint size = 0; InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
            startup.Attributes = Marshal.AllocHGlobal((nint)size);
            Check(InitializeProcThreadAttributeList(startup.Attributes, 1, 0, ref size)); initialized = true;
            Check(UpdateProcThreadAttribute(startup.Attributes, 0, 0x20016, console, (nuint)IntPtr.Size, IntPtr.Zero, IntPtr.Zero));
            var command = new StringBuilder(Quote(path) + " " + string.Join(' ', arguments.Select(Quote)));
            Check(CreateProcess(path, command, IntPtr.Zero, IntPtr.Zero, false, 0x00080004, IntPtr.Zero, cwd, ref startup, out var info));
            process = new(info.Process, true); thread = new(info.Thread, true);
            Check(AssignProcessToJobObject(job, process));
            var tracked = Process.GetProcessById((int)info.Id); _ = tracked.SafeHandle;
            Check(ResumeThread(thread) != uint.MaxValue);
            return new WindowsTerminal(tracked, job, console, write, outputRead);
        }
        catch
        {
            if (process is not null) TerminateProcess(process, 1);
            job?.Dispose(); write?.Dispose(); outputRead?.Dispose(); if (console != IntPtr.Zero) ClosePseudoConsole(console); throw;
        }
        finally
        {
            read?.Dispose(); outputWrite?.Dispose(); process?.Dispose(); thread?.Dispose();
            if (initialized) DeleteProcThreadAttributeList(startup.Attributes);
            if (startup.Attributes != IntPtr.Zero) Marshal.FreeHGlobal(startup.Attributes);
        }
    }
    public override void Resize(int columns, int rows)
    { Dimensions(columns, rows); Marshal.ThrowExceptionForHR(ResizePseudoConsole(console, new() { X = (short)columns, Y = (short)rows })); }
    public override Task TerminateAsync() => termination ??= Terminate();
    private async Task Terminate()
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        await WindowsProcess.TerminateAndWait(job, deadline.Token);
        await Completion.WaitAsync(deadline.Token);
        var handle = Interlocked.Exchange(ref console, IntPtr.Zero);
        if (handle != IntPtr.Zero) await Task.Run(() => ClosePseudoConsole(handle)).WaitAsync(TimeSpan.FromSeconds(5));
    }
    public override async ValueTask DisposeAsync()
    { try { await TerminateAsync(); } finally { input.Dispose(); output.Dispose(); job.Dispose(); process.Dispose(); } }
    private static string Quote(string value) => "\"" + System.Text.RegularExpressions.Regex.Replace(value, "(\\\\*)(\"|$)",
        match => new string('\\', match.Groups[1].Length * 2) + (match.Groups[2].Length == 0 ? "" : "\\\"")) + "\"";
    private static void Check(bool success) { if (!success) throw new Win32Exception(Marshal.GetLastPInvokeError()); }
}
