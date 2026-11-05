using Fpas.Cli.Diagnostics;
using Fpas.Compiler;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Formatting;

internal sealed record FormatFile(string Path, bool Changed, bool Written);
internal sealed record FormatError(string Code, string Message);
internal sealed record FormatReport(bool Success, string Mode, IReadOnlyList<FormatFile> Files,
    IReadOnlyList<CompilerDiagnostic> Diagnostics, FormatError? Error = null)
{
    internal void Write(bool json, bool list)
    {
        if (json) { Console.WriteLine(JsonSerializer.Serialize(this, FormatJson.Default.FormatReport)); return; }
        DiagnosticRenderer.Write(Diagnostics);
        if (Error is { } error) Console.Error.WriteLine(DiagnosticRenderer.Safe(error.Code + ": " + error.Message));
        if (list)
            foreach (var file in Files.Where(file => file.Changed)) Console.WriteLine(file.Path);
        else if (Mode == "check" && Files.Any(file => file.Changed))
        {
            Console.Error.WriteLine("Formatting would change:");
            foreach (var file in Files.Where(file => file.Changed)) Console.Error.WriteLine("  " + DiagnosticRenderer.Safe(file.Path));
            Console.Error.WriteLine("  help: Run fpas fmt with the same paths to rewrite them.");
        }
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull, Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(FormatReport))]
internal partial class FormatJson : JsonSerializerContext;
