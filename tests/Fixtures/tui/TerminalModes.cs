using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text.Json;
using Fpas.Tui;
using Fpas.Ui;
using Microsoft.Win32.SafeHandles;
using Mvu = Fpas.Mvu.Application;

var fail = args[0] == "failure";
using var terminal = new NativeTerminal();
var before = terminal.Read();
Modes? active = null;
Exception? failure = null;
try
{
    Application.Run<int, int>(0, Update, View);
}
catch (Exception error)
{
    failure = error;
}

File.WriteAllText("terminal-modes.json", JsonSerializer.Serialize(new
{
    Before = before,
    Active = active,
    After = terminal.Read(),
    Error = failure?.Message
}));

Mvu.Transition<int, int> Update(int state, int message)
{
    active = terminal.Read();
    if (!fail) return new(state, [Mvu.Exit<int>()]);
    terminal.DisconnectOutput();
    return new(message, []);
}

static Elements.Element<int> View(int state) =>
    state == 0 ? Elements.Button<int>("advance", "terminal-ready", 1) : Elements.Text<int>("next frame");

sealed record Modes(string Input, uint? Output);

sealed class NativeTerminal : IDisposable
{
    private readonly nint input, output;
    private readonly SafeFileHandle? inspection;

    internal NativeTerminal()
    {
        if (!OperatingSystem.IsWindows()) return;
        input = GetStdHandle(-10);
        output = GetStdHandle(-11);
        var process = GetCurrentProcess();
        Check(DuplicateHandle(process, output, process, out var handle, 0, false, 2));
        inspection = handle;
        // Make the output mode change observable even if the console already uses VT.
        Check(GetConsoleMode(output, out var mode));
        Check(SetConsoleMode(output, mode & ~4u));
    }

    internal Modes Read()
    {
        if (inspection is { } handle)
        {
            Check(GetConsoleMode(input, out var inputMode));
            Check(GetConsoleMode(handle.DangerousGetHandle(), out var outputMode));
            return new(inputMode.ToString("X"), outputMode);
        }

        var mode = new byte[256];
        Check(TcGetAttr(0, mode) == 0);
        return new(Convert.ToHexString(mode), null);
    }

    internal void DisconnectOutput()
    {
        if (OperatingSystem.IsWindows())
        {
            Check(CloseHandle(output));
            return;
        }

        // The runtime can duplicate stdout. Break its terminal aliases while retaining stdin and stderr.
        var device = new FileInfo("/proc/self/fd/1").LinkTarget ?? throw new IOException("Missing terminal device.");
        var outputs = Directory.EnumerateFileSystemEntries("/proc/self/fd")
            .Select(path => new FileInfo(path))
            .Where(file => file.Name is not ("0" or "2") && file.LinkTarget == device)
            .Select(file => int.Parse(file.Name))
            .ToArray();
        using var full = File.OpenHandle("/dev/full", FileMode.Open, FileAccess.Write);
        foreach (var descriptor in outputs)
            Check(Dup2((int)full.DangerousGetHandle(), descriptor) == descriptor);
    }

    public void Dispose() => inspection?.Dispose();

    private static void Check(bool success)
    {
        if (!success) throw new Win32Exception(Marshal.GetLastPInvokeError());
    }

    [DllImport("kernel32.dll")] private static extern nint GetCurrentProcess();
    [DllImport("kernel32.dll")] private static extern nint GetStdHandle(int standard);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool CloseHandle(nint handle);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool DuplicateHandle(nint sourceProcess, nint source, nint targetProcess, out SafeFileHandle target, uint access, bool inherit, uint options);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool GetConsoleMode(nint handle, out uint mode);
    [DllImport("kernel32.dll", SetLastError = true)] private static extern bool SetConsoleMode(nint handle, uint mode);
    [DllImport("libc", EntryPoint = "tcgetattr", SetLastError = true)] private static extern int TcGetAttr(int fd, [Out] byte[] mode);
    [DllImport("libc", EntryPoint = "dup2", SetLastError = true)] private static extern int Dup2(int source, int target);
}
