using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Fpas.Debugging.Processes;

internal sealed class OwnedBackend : IAsyncDisposable
{
    private readonly Process process;
    private readonly IDisposable? job;
    private bool disposed;
    internal Stream Input { get; }
    internal Stream Output { get; }
    internal Stream Error { get; }

    private OwnedBackend(Process process, Stream input, Stream output, Stream error, IDisposable? job = null)
    {
        this.process = process; Input = input; Output = output; Error = error; this.job = job;
    }

    internal static void Validate(string path)
    {
        if (!Path.IsPathFullyQualified(path) || !File.Exists(path))
            throw new ArgumentException("--netcoredbg requires an absolute path to an existing executable.");
        if (!OperatingSystem.IsWindows() && !OperatingSystem.IsLinux())
            throw new PlatformNotSupportedException("Agent debugging supports Linux and Windows.");
        if (OperatingSystem.IsLinux())
        {
            var mode = File.GetUnixFileMode(path);
            if ((mode & (UnixFileMode.UserExecute | UnixFileMode.GroupExecute | UnixFileMode.OtherExecute)) == 0)
                throw new ArgumentException("The selected NetCoreDbg file is not executable. Set its execute permission.");
            if (!File.Exists("/usr/bin/setsid") && !File.Exists("/bin/setsid"))
                throw new PlatformNotSupportedException("Linux agent debugging requires setsid (util-linux).");
        }
    }

    internal static OwnedBackend Start(string path, string cwd)
    {
        Validate(path);
        if (OperatingSystem.IsWindows())
        {
            var backend = WindowsBackend.Start(path, cwd);
            return new(backend.Process, backend.Input, backend.Output, backend.Error, backend.Job);
        }
        if (Prctl(36, 1, 0, 0, 0) != 0)
            throw new Win32Exception(Marshal.GetLastPInvokeError(), "Unable to establish child reaping.");
        var start = new ProcessStartInfo(File.Exists("/usr/bin/setsid") ? "/usr/bin/setsid" : "/bin/setsid")
        {
            UseShellExecute = false, WorkingDirectory = cwd,
            RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true
        };
        start.ArgumentList.Add("--"); start.ArgumentList.Add(path); start.ArgumentList.Add("--interpreter=vscode");
        var process = Process.Start(start) ?? throw new IOException("Unable to start NetCoreDbg.");
        return new(process, process.StandardInput.BaseStream, process.StandardOutput.BaseStream, process.StandardError.BaseStream);
    }

    internal bool HasExited => process.HasExited;
    internal int Id => process.Id;
    internal Task WaitForExit(CancellationToken token) => process.WaitForExitAsync(token);

    public async ValueTask DisposeAsync()
    {
        if (disposed) return;
        disposed = true;
        try
        {
            if (OperatingSystem.IsWindows()) job!.Dispose();
            else
            {
                // The setsid launcher establishes this group before executing any backend code.
                if (Kill(-process.Id, 9) != 0 && Marshal.GetLastPInvokeError() != 3)
                    throw new Win32Exception(Marshal.GetLastPInvokeError(), "Unable to terminate the owned backend group.");
                try { if (!process.HasExited) process.Kill(entireProcessTree: true); }
                catch (InvalidOperationException) when (process.HasExited) { }
            }
            Input.Dispose(); Output.Dispose(); Error.Dispose();
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
            await process.WaitForExitAsync(deadline.Token);
            if (OperatingSystem.IsLinux())
            {
                // Descendants become our children even when the adapter exits without a process event.
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
        finally { job?.Dispose(); process.Dispose(); }
    }

    [DllImport("libc", EntryPoint = "kill", SetLastError = true)]
    private static extern int Kill(int pid, int signal);
    [DllImport("libc", EntryPoint = "prctl", SetLastError = true)]
    private static extern int Prctl(int option, nuint a2, nuint a3, nuint a4, nuint a5);
    [DllImport("libc", EntryPoint = "waitpid", SetLastError = true)]
    private static extern int WaitPid(int pid, out int status, int options);
}
