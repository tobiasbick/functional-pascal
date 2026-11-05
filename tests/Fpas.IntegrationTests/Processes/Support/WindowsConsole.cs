using Microsoft.Win32.SafeHandles;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

namespace Fpas.IntegrationTests.Processes.Support;

// A real, isolated Windows console; redirected protocol streams still receive console control events.
internal sealed partial class WindowsConsole : IAsyncDisposable
{
    private readonly Process process;
    private readonly SafeFileHandle job;
    private readonly FileStream terminalInput;
    private readonly Task terminalOutput;
    private readonly StringBuilder transcript = new();
    private IntPtr console;
    internal Stream Input { get; }
    internal Stream Output { get; }
    internal Stream Error { get; }
    internal bool HasExited => process.HasExited;
    internal int ExitCode => process.ExitCode;
    internal string Transcript { get { lock (transcript) return transcript.ToString(); } }

    private WindowsConsole(Process process, SafeFileHandle job, IntPtr console, SafeFileHandle input, SafeFileHandle output,
        Stream? redirectedInput, Stream? redirectedOutput, Stream? redirectedError)
    {
        this.process = process; this.job = job; this.console = console;
        terminalInput = new(input, FileAccess.Write);
        Input = redirectedInput ?? terminalInput; Output = redirectedOutput ?? Stream.Null; Error = redirectedError ?? Stream.Null;
        terminalOutput = Task.Run(() =>
        {
            using var reader = new StreamReader(new FileStream(output, FileAccess.Read), Encoding.UTF8);
            var buffer = new char[4096];
            while (true)
            {
                var count = reader.Read(buffer, 0, buffer.Length);
                if (count == 0) break;
                lock (transcript)
                {
                    if (transcript.Length + count > 4 * 1024 * 1024) throw new IOException("Console transcript exceeded its test limit.");
                    transcript.Append(buffer, 0, count);
                }
            }
        });
    }

    internal static WindowsConsole Start(string path, IReadOnlyList<string> arguments, string cwd, bool capture = true,
        IReadOnlyDictionary<string, string>? environment = null)
    {
        SafeFileHandle? inputRead = null, terminalWrite = null, terminalOutputRead = null, outputWrite = null, job = null;
        IntPtr console = IntPtr.Zero;
        var startup = new StartupInfoEx(); startup.Startup.Size = Marshal.SizeOf<StartupInfoEx>();
        // NULL standard handles request ConPTY handles instead of inheriting the test runner's pipes.
        startup.Startup.Flags = 0x100;
        var handles = new List<SafeFileHandle>();
        var streams = new List<Stream>();
        Process? tracked = null; SafeFileHandle? process = null, thread = null;
        IntPtr handleList = IntPtr.Zero, environmentBlock = IntPtr.Zero;
        var initialized = false;
        try
        {
            Check(CreatePipe(out inputRead, out terminalWrite, IntPtr.Zero, 0));
            Check(CreatePipe(out terminalOutputRead, out outputWrite, IntPtr.Zero, 0));
            Marshal.ThrowExceptionForHR(CreatePseudoConsole(new() { X = 240, Y = 40 }, inputRead, outputWrite, 0, out console));
            job = CreateJobObject(IntPtr.Zero, null);
            Check(!job.IsInvalid);
            var limits = new ExtendedLimits(); limits.Basic.Flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            Check(SetInformationJobObject(job, 9, ref limits, Marshal.SizeOf<ExtendedLimits>()));
            nuint size = 0;
            InitializeProcThreadAttributeList(IntPtr.Zero, capture ? 2 : 1, 0, ref size);
            startup.Attributes = Marshal.AllocHGlobal((nint)size);
            Check(InitializeProcThreadAttributeList(startup.Attributes, capture ? 2 : 1, 0, ref size)); initialized = true;
            Check(UpdateProcThreadAttribute(startup.Attributes, 0, 0x20016, console, (nuint)IntPtr.Size, IntPtr.Zero, IntPtr.Zero));
            if (capture)
            {
                var security = new SecurityAttributes { Length = Marshal.SizeOf<SecurityAttributes>(), Inherit = true };
                Check(CreateInheritablePipe(out var read, out var write, ref security, 0));
                handles.Add(read); streams.Add(new FileStream(write, FileAccess.Write)); Check(SetHandleInformation(write, 1, 0));
                Check(CreateInheritablePipe(out var output, out var outputChild, ref security, 0));
                handles.Add(outputChild); streams.Add(new FileStream(output, FileAccess.Read)); Check(SetHandleInformation(output, 1, 0));
                Check(CreateInheritablePipe(out var error, out var errorChild, ref security, 0));
                handles.Add(errorChild); streams.Add(new FileStream(error, FileAccess.Read)); Check(SetHandleInformation(error, 1, 0));
                startup.Startup.Input = read.DangerousGetHandle(); startup.Startup.Output = outputChild.DangerousGetHandle(); startup.Startup.Error = errorChild.DangerousGetHandle();
                handleList = Marshal.AllocHGlobal(3 * IntPtr.Size);
                for (var i = 0; i < handles.Count; i++) Marshal.WriteIntPtr(handleList, i * IntPtr.Size, handles[i].DangerousGetHandle());
                Check(UpdateProcThreadAttribute(startup.Attributes, 0, 0x20002, handleList, (nuint)(3 * IntPtr.Size), IntPtr.Zero, IntPtr.Zero));
            }
            var variables = new SortedDictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            foreach (System.Collections.DictionaryEntry item in Environment.GetEnvironmentVariables()) variables[(string)item.Key] = (string)item.Value!;
            variables["MSBUILDDISABLENODEREUSE"] = "1"; variables["DOTNET_CLI_USE_MSBUILD_SERVER"] = "0";
            foreach (var (key, value) in environment ?? new Dictionary<string, string>()) variables[key] = value;
            environmentBlock = Marshal.StringToHGlobalUni(string.Concat(variables.Select(pair => pair.Key + "=" + pair.Value + '\0')) + '\0');
            // Keep the test launcher independent of the production Windows process backend.
            var command = new StringBuilder(Quote(path) + " " + string.Join(' ', arguments.Select(Quote)));
            Check(CreateProcess(path, command, IntPtr.Zero, IntPtr.Zero, capture, 0x00080404, environmentBlock, cwd, ref startup, out var info));
            process = new(info.Process, true); thread = new(info.Thread, true);
            Check(AssignProcessToJobObject(job, process));
            tracked = Process.GetProcessById((int)info.Id); _ = tracked.SafeHandle;
            Check(ResumeThread(thread) != uint.MaxValue);
            return new(tracked, job, console, terminalWrite, terminalOutputRead,
                capture ? streams[0] : null, capture ? streams[1] : null, capture ? streams[2] : null);
        }
        catch
        {
            if (process is not null) TerminateProcess(process, 1);
            tracked?.Dispose(); job?.Dispose();
            foreach (var stream in streams) stream.Dispose();
            terminalWrite?.Dispose(); terminalOutputRead?.Dispose();
            if (console != IntPtr.Zero) ClosePseudoConsole(console);
            throw;
        }
        finally
        {
            inputRead?.Dispose(); outputWrite?.Dispose();
            foreach (var handle in handles) handle.Dispose();
            process?.Dispose(); thread?.Dispose();
            if (initialized) DeleteProcThreadAttributeList(startup.Attributes);
            if (startup.Attributes != IntPtr.Zero) Marshal.FreeHGlobal(startup.Attributes);
            if (handleList != IntPtr.Zero) Marshal.FreeHGlobal(handleList);
            if (environmentBlock != IntPtr.Zero) Marshal.FreeHGlobal(environmentBlock);
        }
    }

    internal async Task Interrupt() { await terminalInput.WriteAsync(new byte[] { 3 }); await terminalInput.FlushAsync(); }
    internal Task WaitForExit(CancellationToken token) => process.WaitForExitAsync(token);
    internal Task DrainTerminal() => CloseTerminal();
    internal async Task TerminateAsync()
    {
        Check(TerminateJobObject(job, 1));
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
        await process.WaitForExitAsync(deadline.Token);
    }

    public async ValueTask DisposeAsync()
    {
        try { await TerminateAsync(); }
        finally
        {
            job.Dispose(); Input.Dispose(); Output.Dispose(); Error.Dispose(); terminalInput.Dispose();
            await CloseTerminal(); process.Dispose();
        }
    }

    private async Task CloseTerminal()
    {
        var handle = Interlocked.Exchange(ref console, IntPtr.Zero);
        if (handle != IntPtr.Zero) await Task.Run(() => ClosePseudoConsole(handle)).WaitAsync(TimeSpan.FromSeconds(10));
        await terminalOutput.WaitAsync(TimeSpan.FromSeconds(10));
    }

    private static string Quote(string value)
        => "\"" + System.Text.RegularExpressions.Regex.Replace(value, "(\\\\*)(\"|$)",
            match => new string('\\', match.Groups[1].Length * 2) + (match.Groups[2].Length == 0 ? "" : "\\\"")) + "\"";

    private static void Check(bool success)
    {
        if (!success) throw new Win32Exception(Marshal.GetLastPInvokeError());
    }
}
