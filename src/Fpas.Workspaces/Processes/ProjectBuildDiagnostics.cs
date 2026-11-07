using Fpas.Compiler;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Workspaces.Processes;

public sealed record ProjectBuildDiagnostic(string? Project, string Severity, string? Code, string Message, SourceLocation? Location,
    IReadOnlyList<DiagnosticNote> Related);

internal static class ProjectBuildDiagnostics
{
    internal static async Task<BuildResult> Run(string directory, List<string> arguments, CancellationToken token)
    {
        var assembly = Fpas.Deployment.DistributionAssets.Sidecar(typeof(BuildProcess).Assembly, "Fpas.SdkResolver.dll");
        if (!File.Exists(assembly)) throw new IOException("The MSBuild diagnostic logger is missing from this compiler distribution.");
        var scratch = Directory.CreateTempSubdirectory("fpas-build-");
        try
        {
            var report = Path.Combine(scratch.FullName, "events.jsonl");
            var parameter = Convert.ToBase64String(Encoding.UTF8.GetBytes(report));
            arguments.Add("-logger:Fpas.SdkResolver.Diagnostics.ProjectDiagnosticLogger," + assembly + ";" + parameter);
            var result = await BuildProcess.Run(directory, arguments, token);
            if (!File.Exists(report)) return result;
            var events = new List<(string Context, ProjectBuildDiagnostic Diagnostic)>();
            var details = new Dictionary<(string Context, string? Code, string Message, string? Path, int Line, int Column), IReadOnlyList<DiagnosticNote>>();
            var truncated = false;
            foreach (var line in File.ReadLines(report))
            {
                token.ThrowIfCancellationRequested();
                using var json = JsonDocument.Parse(line);
                var row = json.RootElement;
                var kind = Text(row, "kind");
                if (kind == "limit") { truncated = true; break; }
                var context = Text(row, "context") ?? "";
                if (kind == "details")
                {
                    using var compiler = JsonDocument.Parse(Text(row, "diagnostic")!);
                    var value = compiler.RootElement;
                    if (!value.TryGetProperty("related", out var notes) || notes.ValueKind != JsonValueKind.Array || notes.GetArrayLength() == 0) continue;
                    var location = value.GetProperty("location");
                    details[(context, Text(value, "code"), Text(value, "message") ?? "", Text(location, "path"), Number(location, "line"), Number(location, "column"))] =
                        notes.Deserialize(BuildDiagnosticJson.Default.DiagnosticNoteArray) ?? [];
                    continue;
                }
                var project = Text(row, "project");
                var file = Text(row, "file");
                var lineNumber = Number(row, "line"); var column = Number(row, "column");
                var start = lineNumber > 0 && !string.IsNullOrEmpty(file) ? new SourceLocation(file, lineNumber, Math.Max(column, 1), 0,
                    Positive(Number(row, "endLine")), Positive(Number(row, "endColumn"))) : null;
                events.Add((context, new(project, Text(row, "severity")!, Text(row, "code"), Text(row, "message") ?? "", start, [])));
            }
            return result with { DiagnosticsTruncated = truncated, Diagnostics = events.Select(item =>
            {
                var value = item.Diagnostic;
                var at = value.Location;
                var related = details.GetValueOrDefault((item.Context, value.Code, value.Message, at?.Path, at?.Line ?? 0, at?.Column ?? 0));
                return related is null ? value : value with { Related = related };
            }).Distinct().ToArray() };
        }
        finally { scratch.Delete(recursive: true); }
    }

    private static int? Positive(int value) => value > 0 ? value : null;
    private static string? Text(JsonElement value, string key) => value.ValueKind == JsonValueKind.Object &&
        value.TryGetProperty(key, out var text) && text.ValueKind == JsonValueKind.String ? text.GetString() : null;
    private static int Number(JsonElement value, string key) => value.ValueKind == JsonValueKind.Object &&
        value.TryGetProperty(key, out var number) && number.TryGetInt32(out var result) ? result : 0;
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(DiagnosticNote[]))]
internal partial class BuildDiagnosticJson : JsonSerializerContext;
