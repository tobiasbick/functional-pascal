using Microsoft.Build.Framework;
using Microsoft.Build.Utilities;
using System.Text.Json;

namespace Fpas.Build.Tasks.Diagnostics;

internal static class BuildDiagnostics
{
    internal static void Report(TaskLoggingHelper log, JsonElement diagnostic)
    {
        var code = Text(diagnostic, "code");
        var message = Text(diagnostic, "message") ?? "FPAS compilation failed.";
        var location = diagnostic.GetProperty("location");
        var file = Text(location, "path") ?? Text(diagnostic, "inputPath");
        var line = Number(location, "line"); var column = Number(location, "column");
        var endLine = Number(location, "endLine"); var endColumn = Number(location, "endColumn");
        if (Text(diagnostic, "severity") == "warning")
            log.LogWarning(null, code, Text(diagnostic, "helpLink"), file, line, column, endLine, endColumn, message);
        else log.LogError(null, code, Text(diagnostic, "helpLink"), file, line, column, endLine, endColumn, message);
        log.LogMessage(MessageImportance.Low, "FPAS-DIAGNOSTIC {0}", diagnostic.GetRawText());
        Note("note", Text(diagnostic, "explanation"));
        Note("help", Text(diagnostic, "help"));
        if (diagnostic.TryGetProperty("related", out var related))
            foreach (var item in related.EnumerateArray())
            {
                var at = item.GetProperty("location");
                var prefix = Text(at, "path") is { } path ? $"{path}({Number(at, "line")},{Number(at, "column")}): " : "";
                Note("note", prefix + Text(item, "message"));
            }
        if (diagnostic.TryGetProperty("fixes", out var fixes))
            foreach (var fix in fixes.EnumerateArray())
                Note(Text(fix, "applicability") == "automatic" ? "fix" : "suggestion", Text(fix, "title"));

        // Notes are messages, not additional error-list entries.
        void Note(string label, string? text)
        {
            if (text is not null) log.LogMessage(MessageImportance.High, "  {0}: {1}", label, text);
        }
    }

    private static string? Text(JsonElement value, string name)
        => value.ValueKind == JsonValueKind.Object && value.TryGetProperty(name, out var text) && text.ValueKind == JsonValueKind.String ? text.GetString() : null;
    private static int Number(JsonElement value, string name)
        => value.ValueKind == JsonValueKind.Object && value.TryGetProperty(name, out var number) && number.ValueKind == JsonValueKind.Number && number.TryGetInt32(out var result) ? result : 0;
}
