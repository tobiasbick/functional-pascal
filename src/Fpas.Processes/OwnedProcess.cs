using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Fpas.Processes;

public enum ProcessStdio { Capture, Inherit, CaptureOutput }

public sealed class OwnedProcess : IAsyncDisposable
{
    private readonly Process process;
    private readonly SafeFileHandle? job;
    private readonly object gate = new();
    private Task? disposal;
    private Task? termination;
    public Stream Input { get; }
    public Stream Output { get; }
    public Stream Error { get; }

    private OwnedProcess(Process process, Stream input, Stream output, Stream error, SafeFileHandle? job = null)
    {
        this.process = process; Input = input; Output = output; Error = error; this.job = job;
    }

    public static void Validate(string path)
    {
        if (!Path.IsPathFullyQualified(path) || !File.Exists(path))
            throw new ArgumentException("An absolute path to an existing executable is required.");
        if (!OperatingSystem.IsWindows() && !OperatingSystem.IsLinux())
            throw new PlatformNotSupportedException("Owned processes support Linux and Windows.");
        if (OperatingSystem.IsLinux())
        {
            var mode = File.GetUnixFileMode(path);
            if ((mode & (UnixFileMode.UserExecute | UnixFileMode.GroupExecute | UnixFileMode.OtherExecute)) == 0)
                throw new ArgumentException("The selected file is not executable. Set its execute permission.");
            if (!File.Exists("/usr/bin/setsid") && !File.Exists("/bin/setsid"))
                throw new PlatformNotSupportedException("Linux process ownership requires setsid (util-linux).");
        }
    }

    public static OwnedProcess Start(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment = null, ProcessStdio stdio = ProcessStdio.Capture)
    {
        Validate(path);
        if (stdio is not (ProcessStdio.Capture or ProcessStdio.Inherit or ProcessStdio.CaptureOutput)) throw new ArgumentOutOfRangeException(nameof(stdio));
        if (arguments.Any(argument => argument is null || argument.Contains('\0'))) throw new ArgumentException("Invalid process argument.");
        if (OperatingSystem.IsWindows())
        {
            var backend = WindowsProcess.Start(path, arguments, cwd, environment, stdio);
            return new(backend.Process, backend.Input, backend.Output, backend.Error, backend.Job);
        }
        if (Prctl(36, 1, 0, 0, 0) != 0)
            throw new Win32Exception(Marshal.GetLastPInvokeError(), "Unable to establish child reaping.");
        var start = new ProcessStartInfo(File.Exists("/usr/bin/setsid") ? "/usr/bin/setsid" : "/bin/setsid")
        {
            UseShellExecute = false, WorkingDirectory = cwd,
            RedirectStandardInput = stdio == ProcessStdio.Capture,
            RedirectStandardOutput = stdio != ProcessStdio.Inherit, RedirectStandardError = stdio != ProcessStdio.Inherit
        };
        start.ArgumentList.Add("--"); start.ArgumentList.Add(path);
        foreach (var argument in arguments) start.ArgumentList.Add(argument);
        foreach (var (key, value) in environment ?? new Dictionary<string, string>()) start.Environment[key] = value;
        var process = Process.Start(start) ?? throw new IOException("Unable to start the owned process.");
        return new(process, start.RedirectStandardInput ? process.StandardInput.BaseStream : Stream.Null,
            start.RedirectStandardOutput ? process.StandardOutput.BaseStream : Stream.Null, start.RedirectStandardError ? process.StandardError.BaseStream : Stream.Null);
    }

    public bool HasExited => process.HasExited;
    public int Id => process.Id;
    public int ExitCode => process.ExitCode;
    public Task WaitForExit(CancellationToken token) => process.WaitForExitAsync(token);

    public ValueTask DisposeAsync()
    {
        lock (gate) return new(disposal ??= DisposeCore());
    }

    public Task TerminateAsync()
    {
        lock (gate) return termination ??= TerminateCore();
    }

    private async Task DisposeCore()
    {
        try { await TerminateAsync(); }
        finally
        {
            job?.Dispose();
            Input.Dispose(); Output.Dispose(); Error.Dispose();
            process.Dispose();
        }
    }

    private async Task TerminateCore()
    {
        try
        {
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
            if (OperatingSystem.IsWindows()) await WindowsProcess.TerminateAndWait(job!, deadline.Token);
            else
            {
                // The setsid launcher establishes this group before executing any target code.
                if (Kill(-process.Id, 9) != 0 && Marshal.GetLastPInvokeError() != 3)
                    throw new Win32Exception(Marshal.GetLastPInvokeError(), "Unable to terminate the owned process group.");
                try { if (!process.HasExited) process.Kill(entireProcessTree: true); }
                catch (InvalidOperationException) when (process.HasExited) { }
            }
            await process.WaitForExitAsync(deadline.Token);
            if (OperatingSystem.IsLinux())
            {
                // Adopt and reap descendants even when the original parent exits first.
                while (true)
                {
                    var child = WaitPid(-process.Id, out _, 1);
                    if (child > 0) continue;
                    if (child < 0 && Marshal.GetLastPInvokeError() == 10) break;
                    deadline.Token.ThrowIfCancellationRequested();
                    await Task.Delay(10, deadline.Token);
                }
            }
        }
        finally { job?.Dispose(); }
    }

    [DllImport("libc", EntryPoint = "kill", SetLastError = true)]
    private static extern int Kill(int pid, int signal);
    [DllImport("libc", EntryPoint = "prctl", SetLastError = true)]
    private static extern int Prctl(int option, nuint a2, nuint a3, nuint a4, nuint a5);
    [DllImport("libc", EntryPoint = "waitpid", SetLastError = true)]
    private static extern int WaitPid(int pid, out int status, int options);
}
