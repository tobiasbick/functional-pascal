using System.Text.Json;
using System.Text.Json.Serialization;
using Fpas.Workspaces.Processes;

namespace Fpas.Cli.Reporting;

internal sealed record OperationError(string Code, string Message);
internal sealed record OperationReport(string Command, bool Success, string Status)
{
    public int SchemaVersion => 1;
    public string? Artifact { get; init; }
    public BuildResult? Process { get; init; }
    public Profiling.TraceSummary? Trace { get; init; }
    public Terminal.TerminalResult? Terminal { get; init; }
    public OperationError? Error { get; init; }

    internal int Write(int exit)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(this, OperationJson.Default.OperationReport);
        if (bytes.Length > 4_194_304)
        {
            bytes = JsonSerializer.SerializeToUtf8Bytes(this with { Success = false, Status = "failed", Process = null, Trace = null, Terminal = null,
                Error = new("report_limit", "Report exceeded 4 MiB; inspect the retained artifact.") }, OperationJson.Default.OperationReport);
            exit = 1;
        }
        Console.OpenStandardOutput().Write(bytes); Console.WriteLine(); return exit;
    }
    internal static int Fail(string command, string code, string message, int exit = 1)
        => new OperationReport(command, false, code is "cancelled" or "timeout" ? code : "failed") { Error = new(code, message) }.Write(exit);
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(OperationReport))]
[JsonSerializable(typeof(Terminal.TerminalScript))]
[JsonSerializable(typeof(Profiling.TraceSummary))]
internal partial class OperationJson : JsonSerializerContext;
