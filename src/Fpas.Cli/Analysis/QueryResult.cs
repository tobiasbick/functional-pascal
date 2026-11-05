using Fpas.Workspaces.Queries;
using Fpas.Cli.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed record QueryPayload(DefinitionResult? Definition, TypeResult? Type, ReferencesResult? Refs, FindResult? Find, InspectResult? Inspect,
    CalleesResult? Callees, CallersResult? Callers);
internal sealed record QueryResult(string Status, string? Reason, string? Context, string Configuration, string? SnapshotId, QuerySelection? Selection,
    IReadOnlyList<AnalysisDiagnostic> Diagnostics, AnalysisCoverage Coverage, Preparation? Preparation, bool Truncated,
    IReadOnlyList<string> BinaryBoundaries, QueryPayload? Result, AnalysisError? Error)
{
    internal static QueryResult From(CheckResult check, SourceQueryResult result)
        => new(result.Status, result.Reason, check.Context, check.Configuration, check.SnapshotId, result.Selection, check.Diagnostics, check.Coverage with { Searched = result.Searched ?? check.Coverage.Analyzed }, check.Preparation,
            result.Truncated, result.BinaryBoundaries, result.Status == "ok" ? new(result.Definition, result.Type, result.Refs, result.Find, result.Inspect, result.Callees, result.Callers) : null, null);

    internal static QueryResult Failure(string? context, string configuration, string code, string message, CheckResult? check = null, bool truncated = false, IReadOnlyList<string>? candidates = null)
        => new(code == "cancelled" ? "cancelled" : "failed", null, context, configuration, null, null,
            check?.Diagnostics ?? [], check is not null ? check.Coverage with { Complete = code is "invalid_argument" or "ambiguous_project" && check.Coverage.Complete } : new(false, [], [], []),
            check?.Preparation, false, [], null, new(code, message, truncated, candidates));

    internal async Task<int> Report(int exit)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(this, QueryJson.Default.QueryResult);
        if (bytes.Length > 4_194_304)
        {
            bytes = JsonSerializer.SerializeToUtf8Bytes(Failure(Context, Configuration, "result_limit", "Query response exceeds 4 MiB."), QueryJson.Default.QueryResult);
            exit = 1;
        }
        await Console.OpenStandardOutput().WriteAsync(bytes);
        Console.WriteLine(); return exit;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(QueryResult))]
internal partial class QueryJson : JsonSerializerContext;
