// [Documentation](../../../docs/pascal/program-structure/cli.md#query-checked-sources)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Loading;
using Fpas.Cli.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed record QueryPayload(DefinitionResult? Definition, TypeResult? Type, ReferencesResult? Refs, FindResult? Find, InspectResult? Inspect,
    CalleesResult? Callees, CallersResult? Callers, EditorCompletion? Complete, EditorCallSignatures? Signature)
{
    internal static QueryPayload From(SourceQueryResult result)
        => new(result.Definition, result.Type, result.Refs, result.Find, result.Inspect, result.Callees, result.Callers, result.Complete, result.Signature);
}
internal sealed record QueryResult(string Status, string? Reason, string? Context, string Configuration, string? SnapshotId, QuerySelection? Selection,
    IReadOnlyList<AnalysisDiagnostic> Diagnostics, AnalysisCoverage Coverage, Preparation? Preparation, bool Truncated,
    IReadOnlyList<string> BinaryBoundaries, QueryPayload? Result, AnalysisError? Error)
{
    /// <summary>Gets the requested diagnostic window and its counts.</summary>
    public DiagnosticPage? DiagnosticPage { get; init; }

    internal const int OutputLimit = 4 * 1024 * 1024;

    internal static QueryResult From(CheckResult check, SourceQueryResult result)
        => new(result.Status, result.Reason, check.Context, check.Configuration, check.SnapshotId, result.Selection, check.Diagnostics,
            check.Coverage with { Searched = result.Searched ?? check.Coverage.Analyzed, Semantics = result.Coverage?.Projects }, check.Preparation,
            result.Truncated, result.BinaryBoundaries, result.Status == "ok" ? QueryPayload.From(result) : null, null) { DiagnosticPage = check.DiagnosticPage };

    internal static QueryResult Failure(string? context, string configuration, string code, string message, CheckResult? check = null, bool truncated = false, IReadOnlyList<string>? candidates = null)
        => new(code == "cancelled" ? "cancelled" : "failed", null, context, configuration, null, null,
            check?.Diagnostics ?? [], check is not null ? check.Coverage with { Complete = code is "invalid_argument" or "ambiguous_project" && check.Coverage.Complete } : new(false, [], [], []),
            check?.Preparation, false, [], null, new(code, message, truncated, candidates)) { DiagnosticPage = check?.DiagnosticPage };

    internal async Task<int> Report(int exit)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(this, QueryJson.Default.QueryResult);
        if (bytes.Length > OutputLimit)
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
[JsonSerializable(typeof(QueryBatchReport))]
[JsonSerializable(typeof(QueryBatchItem))]
[JsonSerializable(typeof(QueryRequest))]
internal partial class QueryJson : JsonSerializerContext;

internal sealed record QueryBatchItem(string? Operation, string Status, string? Reason, QuerySelection? Selection, bool Truncated,
    IReadOnlyList<string> BinaryBoundaries, IReadOnlyList<string>? Searched, QueryPayload? Result, AnalysisError? Error)
{
    /// <summary>Gets the returned declarations from an ambiguous dependency.</summary>
    public IReadOnlyList<DeclarationMatch>? Candidates { get; init; }
    /// <summary>Gets the total checked declarations from an ambiguous dependency.</summary>
    public int? CandidateCount { get; init; }

    internal static QueryBatchItem From(string operation, SourceQueryResult result)
        => new(operation, result.Status, result.Reason, result.Selection, result.Truncated, result.BinaryBoundaries, result.Searched,
            result.Status == "ok" ? QueryPayload.From(result) : null, null);

    internal static QueryBatchItem Failure(string? operation, ContextFailure failure)
        => new(operation, "failed", null, null, false, [], null, null, AnalysisError.From(failure));
}

internal sealed record QueryBatchReport(string Status, string? Context, string Configuration, string? SnapshotId,
    IReadOnlyList<AnalysisDiagnostic> Diagnostics, AnalysisCoverage? Coverage, Preparation? Preparation,
    IReadOnlyList<QueryBatchItem> Results, AnalysisError? Error)
{
    /// <summary>Gets the version of the batch report layout.</summary>
    public int SchemaVersion { get; init; } = 1;
    /// <summary>Gets the diagnostic window shared by every query in the batch.</summary>
    public DiagnosticPage? DiagnosticPage { get; init; }
    /// <summary>Gets the draft source paths used for analysis, when supplied.</summary>
    public IReadOnlyList<string>? OverlayFiles { get; init; }

    internal static QueryBatchReport Failure(string? context, string configuration, AnalysisError error, CheckResult? check = null)
        => new(error.Code == "cancelled" ? "cancelled" : "failed", context, configuration, null, check?.Diagnostics ?? [],
            check is null ? null : check.Coverage with { Complete = false }, check?.Preparation, [], error) { DiagnosticPage = check?.DiagnosticPage, OverlayFiles = check?.OverlayFiles };

    internal async Task<int> WriteAsync(int exit)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(this with { Results = [] }, QueryJson.Default.QueryBatchReport);
        int Size(QueryBatchItem item) => JsonSerializer.SerializeToUtf8Bytes(item, QueryJson.Default.QueryBatchItem).Length;
        var overflow = new ContextFailure("result_limit", "This batch entry exceeds the remaining response budget. Request a smaller page or query it separately.");
        var limited = Results.Select(item => QueryBatchItem.Failure(item.Operation, overflow)).ToArray();
        var reserved = limited.Select(Size).ToArray();
        var remaining = (long)QueryResult.OutputLimit - bytes.Length - Math.Max(0, limited.Length - 1) - reserved.Sum();
        if (remaining < 0)
        {
            bytes = JsonSerializer.SerializeToUtf8Bytes(Failure(Context, Configuration, new("result_limit", "Query batch metadata exceeds 4 MiB.")),
                QueryJson.Default.QueryBatchReport);
            exit = 1;
        }
        else if (Results.Count > 0)
        {
            // Reserve one error per entry so oversized items cannot crowd out later answers.
            var retained = Results.ToArray();
            for (var index = 0; index < retained.Length; index++)
            {
                var size = Size(retained[index]) - reserved[index];
                if (size <= remaining) remaining -= size;
                else retained[index] = limited[index];
            }
            bytes = JsonSerializer.SerializeToUtf8Bytes(this with { Results = retained }, QueryJson.Default.QueryBatchReport);
        }
        await Console.OpenStandardOutput().WriteAsync(bytes);
        Console.WriteLine();
        return exit;
    }
}
