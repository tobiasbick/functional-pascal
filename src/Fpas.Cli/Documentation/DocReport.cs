using Fpas.Cli.Analysis;
using Fpas.Cli.Diagnostics;
using Fpas.Workspaces.Documentation;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Documentation;

internal sealed record DocReport(bool Success, string Status, string? Context, string Configuration, string? SnapshotId,
    AnalysisCoverage Coverage, Preparation? Preparation, IReadOnlyList<AnalysisDiagnostic> Diagnostics, DocumentationResult? Result, AnalysisError? Error)
{
    internal static DocReport From(CheckResult check, DocumentationResult result) => new(true, result.Status, check.Context, check.Configuration,
        result.SnapshotId, check.Coverage with { Searched = result.SearchedProjects }, check.Preparation, check.Diagnostics, result, null);
    internal static DocReport Failure(string? context, string configuration, string code, string message, CheckResult? check = null,
        bool truncated = false, IReadOnlyList<string>? candidates = null) => new(false, code == "cancelled" ? "cancelled" : "failed", context, configuration,
        null, check is null ? new(false, [], [], []) : check.Coverage with { Complete = false }, check?.Preparation, check?.Diagnostics ?? [], null, new(code, message, truncated, candidates));
    internal bool Write(bool json)
    {
        if (json)
        {
            var bytes = JsonSerializer.SerializeToUtf8Bytes(this, DocJson.Default.DocReport);
            var limited = bytes.Length > 4_194_304;
            if (limited) bytes = JsonSerializer.SerializeToUtf8Bytes(Failure(Context, Configuration, "result_limit", "Documentation response exceeds 4 MiB."), DocJson.Default.DocReport);
            using var output = Console.OpenStandardOutput(); output.Write(bytes); Console.WriteLine(); return !limited;
        }
        if (Result is { } result)
        {
            foreach (var entry in result.Entries)
            {
                Console.WriteLine(DiagnosticRenderer.Safe(entry.QualifiedName + " [" + entry.Origin + ", " + entry.Assembly + "]"));
                Console.WriteLine(DiagnosticRenderer.Safe(entry.Signature));
                foreach (var line in DocumentationDisplay.Text(entry.Documentation).Replace("\r\n", "\n", StringComparison.Ordinal).Replace('\r', '\n').Split('\n'))
                    Console.WriteLine(DiagnosticRenderer.Safe(line));
                Console.WriteLine();
            }
            Console.WriteLine($"{result.Entries.Count}/{result.TotalCount} entries. Status: {result.Status}." +
                (result.NextOffset is { } next ? $" Next offset: {next}." : "") +
                (result.OmittedDocumentationCount != 0 ? $" {result.OmittedDocumentationCount} documentation blocks omitted by budget." : ""));
        }
        DiagnosticRenderer.Write(Diagnostics);
        foreach (var issue in Preparation?.Issues ?? []) Console.Error.WriteLine(DiagnosticRenderer.Safe(issue.Project + ": " + issue.Code + ": " + issue.Message));
        if (Error is { } error) Console.Error.WriteLine(DiagnosticRenderer.Safe(error.Code + ": " + error.Message));
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(DocReport))]
internal partial class DocJson : JsonSerializerContext;
