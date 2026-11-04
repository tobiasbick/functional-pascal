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
        var command = new Command("debug", "Host stdio MCP debugging tools for an agent. Program selection uses debug_launch; target stdin is unsupported.");
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
