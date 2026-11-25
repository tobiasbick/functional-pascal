// [Documentation](../../docs/pascal/libraries/processes.md#owning-a-process-tree)
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace Fpas.Processes;

/// <summary>Selects how an owned process connects its standard streams.</summary>
public enum ProcessStdio
{
    /// <summary>Redirect input, output and error through pipes.</summary>
    Capture,
    /// <summary>Inherit all standard streams from the current process.</summary>
    Inherit,
    /// <summary>Inherit input and redirect output and error through pipes.</summary>
    CaptureOutput,
    /// <summary>Redirect input through a pipe and inherit output and error.</summary>
    CaptureInput
}

/// <summary>
/// Owns a Windows job or a Linux process group and terminates its members on disposal.
/// On Linux, descendants that enter another process group or session are outside this ownership boundary.
/// </summary>
public sealed class OwnedProcess : IAsyncDisposable
{
    private readonly Process process;
    private readonly SafeFileHandle? job;
    private readonly object gate = new();
    private Task? disposal;
    private Task? termination;
    /// <summary>Gets the writable standard input pipe, or <see cref="Stream.Null"/> when not captured.</summary>
    public Stream Input { get; }
    /// <summary>Gets the readable standard output pipe, or <see cref="Stream.Null"/> when not captured.</summary>
    public Stream Output { get; }
    /// <summary>Gets the readable standard error pipe, or <see cref="Stream.Null"/> when not captured.</summary>
    public Stream Error { get; }

    private OwnedProcess(Process process, Stream input, Stream output, Stream error, SafeFileHandle? job = null)
    {
        this.process = process; Input = input; Output = output; Error = error; this.job = job;
    }

    /// <summary>Checks that an executable can be started as an owned process on this platform.</summary>
    /// <param name="path">The absolute executable path.</param>
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

    /// <summary>Starts an executable inside a new ownership boundary.</summary>
    /// <param name="path">The absolute executable path.</param>
    /// <param name="arguments">The arguments, passed without shell interpretation.</param>
    /// <param name="cwd">The working directory.</param>
    /// <param name="environment">Variables that override the inherited environment.</param>
    /// <param name="stdio">How the standard streams are connected.</param>
    /// <returns>The started process; dispose it to terminate all owned members.</returns>
    public static OwnedProcess Start(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment = null, ProcessStdio stdio = ProcessStdio.Capture)
    {
        Validate(path);
        if (stdio is not (ProcessStdio.Capture or ProcessStdio.Inherit or ProcessStdio.CaptureOutput or ProcessStdio.CaptureInput)) throw new ArgumentOutOfRangeException(nameof(stdio));
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
            RedirectStandardInput = stdio is ProcessStdio.Capture or ProcessStdio.CaptureInput,
            RedirectStandardOutput = stdio is ProcessStdio.Capture or ProcessStdio.CaptureOutput,
            RedirectStandardError = stdio is ProcessStdio.Capture or ProcessStdio.CaptureOutput
        };
        start.ArgumentList.Add("--"); start.ArgumentList.Add(path);
        foreach (var argument in arguments) start.ArgumentList.Add(argument);
        foreach (var (key, value) in environment ?? new Dictionary<string, string>()) start.Environment[key] = value;
        var process = Process.Start(start) ?? throw new IOException("Unable to start the owned process.");
        return new(process, start.RedirectStandardInput ? process.StandardInput.BaseStream : Stream.Null,
            start.RedirectStandardOutput ? process.StandardOutput.BaseStream : Stream.Null, start.RedirectStandardError ? process.StandardError.BaseStream : Stream.Null);
    }

    /// <summary>Gets whether the root process has exited.</summary>
    public bool HasExited => process.HasExited;
    /// <summary>Gets the root process identifier.</summary>
    public int Id => process.Id;
    /// <summary>Gets the root process exit code after it has exited.</summary>
    public int ExitCode => process.ExitCode;
    /// <summary>Waits until the root process exits.</summary>
    /// <param name="token">Cancels the wait without terminating the process.</param>
    /// <returns>A task that completes when the root process exits.</returns>
    public Task WaitForExit(CancellationToken token) => process.WaitForExitAsync(token);

    /// <summary>Writes bytes to redirected stdin and closes it to signal EOF.</summary>
    /// <param name="data">Verbatim input bytes.</param>
    /// <param name="token">Cancels input delivery.</param>
    /// <returns>Whether the bytes could be delivered before the pipe closed.</returns>
    public async Task<bool> SupplyInputAsync(ReadOnlyMemory<byte> data, CancellationToken token)
    {
        if (ReferenceEquals(Input, Stream.Null)) return false;
        try
        {
            using var input = Input;
            if (!data.IsEmpty)
            {
                await input.WriteAsync(data, token);
                await input.FlushAsync(token);
            }
            return true;
        }
        catch (Exception error) when (error is IOException or ObjectDisposedException) { return false; }
    }

    /// <summary>Terminates all owned members and releases the streams and handles.</summary>
    /// <returns>An operation that completes after process cleanup.</returns>
    public ValueTask DisposeAsync()
    {
        lock (gate) return new(disposal ??= DisposeCore());
    }

    /// <summary>Terminates all owned members and waits for them, at most five seconds.</summary>
    /// <returns>A task that completes after the owned members exit.</returns>
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
