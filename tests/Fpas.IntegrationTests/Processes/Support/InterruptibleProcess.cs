using Fpas.Processes;
using System.ComponentModel;
using System.Runtime.InteropServices;

namespace Fpas.IntegrationTests.Processes.Support;

internal sealed class InterruptibleProcess : IAsyncDisposable
{
    private readonly OwnedProcess? group;
    private readonly WindowsConsole? console;
    private InterruptibleProcess(OwnedProcess group) => this.group = group;
    private InterruptibleProcess(WindowsConsole console) => this.console = console;
    internal Stream Input => console?.Input ?? group!.Input;
    internal Stream Output => console?.Output ?? group!.Output;
    internal Stream Error => console?.Error ?? group!.Error;
    internal bool HasExited => console?.HasExited ?? group!.HasExited;
    internal int ExitCode => console?.ExitCode ?? group!.ExitCode;
    internal static InterruptibleProcess Start(string path, IReadOnlyList<string> arguments, string cwd, IReadOnlyDictionary<string, string>? environment = null)
        => OperatingSystem.IsWindows() ? new(WindowsConsole.Start(path, arguments, cwd, environment: environment)) : new(OwnedProcess.Start(path, arguments, cwd, environment));
    internal Task WaitForExit(CancellationToken token) => console?.WaitForExit(token) ?? group!.WaitForExit(token);
    internal Task TerminateAsync() => console?.TerminateAsync() ?? group!.TerminateAsync();
    internal Task Interrupt()
    {
        if (console is not null) return console.Interrupt();
        if (Kill(group!.Id, 2) != 0) throw new Win32Exception(Marshal.GetLastPInvokeError());
        return Task.CompletedTask;
    }
    public ValueTask DisposeAsync() => console?.DisposeAsync() ?? group!.DisposeAsync();
    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int pid, int signal);
}
