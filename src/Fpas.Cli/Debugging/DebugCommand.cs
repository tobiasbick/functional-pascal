using Fpas.Debugging;
using System.CommandLine;

namespace Fpas.Cli.Debugging;

internal static class DebugCommand
{
    internal static Command Create()
    {
        var backend = new Option<string>("--netcoredbg")
        {
            Required = true, Description = "Absolute path to separately installed NetCoreDbg. No automatic download."
        };
        var command = new Command("debug", """
            Host stdio MCP debugging tools for an agent.

            Register this command in the agent host. Select a compiled program with debug_launch.
            Target stdin is unsupported. NetCoreDbg is installed separately.

            Examples:
              fpas debug --netcoredbg <absolute-netcoredbg-path>
            """);
        command.Options.Add(backend);
        command.SetAction(async (result, token) =>
        {
            try { await DebugServer.RunAsync(result.GetValue(backend)!, token); return 0; }
            catch (OperationCanceledException) when (token.IsCancellationRequested) { return 0; }
            catch (Exception exception) when (exception is ArgumentException or IOException or UnauthorizedAccessException or PlatformNotSupportedException or System.ComponentModel.Win32Exception)
            {
                Console.Error.WriteLine("FP6001: " + exception.Message); return 1;
            }
        });
        return command;
    }
}
