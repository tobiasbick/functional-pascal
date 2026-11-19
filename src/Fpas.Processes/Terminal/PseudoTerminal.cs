// [Documentation](../../../docs/pascal/tools/debugging.md#private-terminal-and-tui-control)
namespace Fpas.Processes.Terminal;

/// <summary>A private terminal session with explicit dimensions and owned descendant lifetime.</summary>
public abstract class PseudoTerminal : IAsyncDisposable
{
    /// <summary>Gets the stream that sends keyboard input to the terminal.</summary>
    public abstract Stream Input { get; }
    /// <summary>Gets the stream of terminal output, including VT sequences.</summary>
    public abstract Stream Output { get; }
    /// <summary>Gets the target process identifier.</summary>
    public abstract int Id { get; }
    /// <summary>Gets a task that completes with the target exit code.</summary>
    public abstract Task<int> Completion { get; }
    /// <summary>Changes the terminal dimensions.</summary>
    /// <param name="columns">The column count, 1..500.</param>
    /// <param name="rows">The row count, 1..300.</param>
    public abstract void Resize(int columns, int rows);
    /// <summary>Terminates the target and all owned descendants.</summary>
    public abstract Task TerminateAsync();
    /// <summary>Terminates the target and releases the terminal.</summary>
    public abstract ValueTask DisposeAsync();
    internal virtual void Resume() => throw new InvalidOperationException("This terminal has no suspended target.");

    /// <summary>Starts an executable in a new private terminal.</summary>
    /// <param name="executable">The absolute executable path.</param>
    /// <param name="arguments">The arguments, passed without shell interpretation.</param>
    /// <param name="directory">The existing working directory.</param>
    /// <param name="columns">The initial column count, 1..500.</param>
    /// <param name="rows">The initial row count, 1..300.</param>
    /// <returns>The running terminal session.</returns>
    public static PseudoTerminal Start(string executable, IReadOnlyList<string> arguments, string directory, int columns = 80, int rows = 24)
        => Start(executable, arguments, directory, columns, rows, suspended: false);

    /// <summary>Starts an executable in a new private terminal and holds it before its code runs.</summary>
    /// <param name="executable">The absolute executable path.</param>
    /// <param name="arguments">The arguments, passed without shell interpretation.</param>
    /// <param name="directory">The existing working directory.</param>
    /// <param name="columns">The initial column count, 1..500.</param>
    /// <param name="rows">The initial row count, 1..300.</param>
    /// <returns>The held terminal session; call <see cref="ReleaseStartup"/> to run the target.</returns>
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

    /// <summary>Lets a target started with <see cref="StartSuspended"/> run.</summary>
    public void ReleaseStartup() => Resume();
    internal static void Dimensions(int columns, int rows)
    { if (columns is < 1 or > 500 || rows is < 1 or > 300) throw new ArgumentException("Terminal dimensions require 1..500 columns and 1..300 rows."); }
}
