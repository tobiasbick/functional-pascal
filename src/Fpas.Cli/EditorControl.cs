using System.Runtime.InteropServices;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli;

internal static class EditorControl
{
    internal static async Task<int> RunAsync(string[] command)
    {
        if (!Console.IsInputRedirected || command.Length == 0 ||
            command[0] is not ("target" or "check" or "restore" or "build" or "clean" or "run" or "test" or "fmt"))
            return Reporting.OperationReport.Fail("editor-control", "invalid_argument", "Select a supported captured command and redirect stdin.", 2);
        using var cancelled = new CancellationTokenSource();
        _ = WatchAsync(cancelled);
        return await CliApplication.RunAsync(command, cancelled.Token, editorControlled: true);
    }

    private static async Task WatchAsync(CancellationTokenSource cancelled)
    {
        try { _ = await Console.OpenStandardInput().ReadAsync(new byte[1]); }
        catch (IOException) { }
        try { await cancelled.CancelAsync(); }
        catch (ObjectDisposedException) { }
    }

    internal static async Task<int> RunTerminalAsync(string payload)
    {
        string[]? command;
        try
        {
            if (payload.Length > 32_768) throw new FormatException();
            command = JsonSerializer.Deserialize(Convert.FromBase64String(payload), EditorJson.Default.StringArray);
            if (command is not ["run", ..] || command.Any(value => value is null || value.Contains('\0'))) throw new FormatException();
        }
        catch (Exception error) when (error is FormatException or JsonException)
        { return Reporting.OperationReport.Fail("editor-run", "invalid_argument", "Expected bounded FPAS run arguments.", 2); }
        using var cancelled = new CancellationTokenSource();
        void Interrupt(PosixSignalContext context)
        {
            context.Cancel = true;
            _ = cancelled.CancelAsync();
        }
        using var hangup = OperatingSystem.IsLinux() ? PosixSignalRegistration.Create(PosixSignal.SIGHUP, Interrupt) : null;
        using var terminate = OperatingSystem.IsLinux() ? PosixSignalRegistration.Create(PosixSignal.SIGTERM, Interrupt) : null;
        return await CliApplication.RunAsync(command, cancelled.Token);
    }
}

[JsonSerializable(typeof(string[]))]
internal partial class EditorJson : JsonSerializerContext;
