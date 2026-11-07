using Microsoft.Build.Framework;
using System.Text;
using System.Text.Json;

namespace Fpas.SdkResolver.Diagnostics;

public sealed class ProjectDiagnosticLogger : ILogger
{
    private readonly object gate = new();
    private StreamWriter output = null!;
    private int remaining = 1_048_576;
    public LoggerVerbosity Verbosity { get; set; }
    public string? Parameters { get; set; }

    public void Initialize(IEventSource events)
    {
        var path = Encoding.UTF8.GetString(Convert.FromBase64String(Parameters ?? ""));
        output = new StreamWriter(new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read | FileShare.Delete), new UTF8Encoding(false));
        events.ErrorRaised += (_, e) => Record("error", e.ProjectFile, e.File, e.LineNumber, e.ColumnNumber, e.EndLineNumber, e.EndColumnNumber,
            e.Code, e.Message, e.BuildEventContext);
        events.WarningRaised += (_, e) => Record("warning", e.ProjectFile, e.File, e.LineNumber, e.ColumnNumber, e.EndLineNumber, e.EndColumnNumber,
            e.Code, e.Message, e.BuildEventContext);
        events.MessageRaised += (_, e) =>
        {
            const string prefix = "FPAS-DIAGNOSTIC ";
            if (e.Importance == MessageImportance.Low && e.Message is { } message && message.StartsWith(prefix, StringComparison.Ordinal))
                Write(JsonSerializer.Serialize(new { kind = "details", context = Context(e.BuildEventContext), diagnostic = message[prefix.Length..] }));
        };
    }

    private void Record(string severity, string? project, string? file, int line, int column, int endLine, int endColumn, string? code,
        string? message, BuildEventContext? context) => Write(JsonSerializer.Serialize(new
        { kind = "diagnostic", context = Context(context), severity, project, file, line, column, endLine, endColumn, code, message }));

    private static string Context(BuildEventContext? value) => value is null ? "" : $"{value.NodeId}:{value.ProjectContextId}:{value.TargetId}:{value.TaskId}";

    private void Write(string line)
    {
        lock (gate)
        {
            if (remaining <= 0) return;
            remaining -= Encoding.UTF8.GetByteCount(line) + 2;
            output.WriteLine(remaining > 0 ? line : """{"kind":"limit"}""");
        }
    }

    public void Shutdown() => output?.Dispose();
}
