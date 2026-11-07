namespace Fpas.Processes.Terminal;

// A private terminal session with explicit dimensions and owned descendant lifetime.
public abstract class PseudoTerminal : IAsyncDisposable
{
    public abstract Stream Input { get; }
    public abstract Stream Output { get; }
    public abstract int Id { get; }
    public abstract Task<int> Completion { get; }
    public abstract void Resize(int columns, int rows);
    public abstract Task TerminateAsync();
    public abstract ValueTask DisposeAsync();
    internal virtual void Resume() => throw new InvalidOperationException("This terminal has no suspended target.");

    public static PseudoTerminal Start(string executable, IReadOnlyList<string> arguments, string directory, int columns = 80, int rows = 24)
        => Start(executable, arguments, directory, columns, rows, suspended: false);

    public static PseudoTerminal StartSuspended(string executable, IReadOnlyList<string> arguments, string directory, int columns, int rows)
        => Start(executable, arguments, directory, columns, rows, suspended: true);

    private static PseudoTerminal Start(string executable, IReadOnlyList<string> arguments, string directory, int columns, int rows, bool suspended)
    {
        OwnedProcess.Validate(executable); Dimensions(columns, rows);
        if (!Directory.Exists(directory) || arguments.Any(item => item is null || item.Contains('\0')))
            throw new ArgumentException("Select an existing working directory and arguments without NUL.");
        return OperatingSystem.IsWindows() ? WindowsTerminal.Start(executable, arguments, directory, columns, rows, suspended) :
            LinuxTerminal.Start(executable, arguments, directory, columns, rows, suspended);
    }

    public void ReleaseStartup() => Resume();
    internal static void Dimensions(int columns, int rows)
    { if (columns is < 1 or > 500 || rows is < 1 or > 300) throw new ArgumentException("Terminal dimensions require 1..500 columns and 1..300 rows."); }
}
