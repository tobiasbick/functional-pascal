// [Documentation](../../../docs/pascal/program-structure/cli.md#query-checked-sources)
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.CommandLine;
using System.Text;
using System.Text.Json;

namespace Fpas.Cli.Analysis;

internal sealed class QueryBatchCommand
{
    private const int InputLimit = 256 * 1024;
    internal const int RequestLimit = 64;
    private static readonly UTF8Encoding StrictUtf8 = new(false, true);
    private readonly ContextCommandOptions options = new();
    private readonly DiagnosticPageOptions diagnosticOptions = new();
    private readonly Option<string> overlay = AnalysisOverlay.FileOption();
    private readonly Option<string> expectedSnapshot = SnapshotGuard.CreateOption();

    internal Command Command { get; } = new("batch", """
        Run source queries against one checked snapshot.
        Read 1 to 64 JSON requests from stdin and write one ordered JSON report to stdout.

        Examples:
          fpas query batch Example.fpasproj --existing-inputs < queries.json

          fpas query batch Example.fpasproj --overlay draft.json < queries.json
        """);

    internal QueryBatchCommand()
    {
        options.AddTo(Command);
        diagnosticOptions.AddTo(Command);
        Command.Options.Add(overlay);
        Command.Options.Add(expectedSnapshot);
        Command.SetAction(Execute);
    }

    internal Task<int> ReportArgumentError(ParseResult parse, string message)
        => QueryBatchReport.Failure(CommandArguments.Supplied(parse, options.Context), CommandArguments.Supplied(parse, options.Configuration) ?? "Debug",
            new("invalid_argument", message + " See fpas query batch --help.")).WriteAsync(2);

    private async Task<int> Execute(ParseResult parse, CancellationToken token)
    {
        var input = options.Read(parse);
        var diagnostics = diagnosticOptions.Read(parse);
        var timeout = TimeSpan.FromSeconds(parse.GetValue(options.Timeout));
        using var deadline = new CancellationTokenSource(timeout);
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(token, deadline.Token);
        CheckResult? check = null;
        try
        {
            using var document = await ReadRequestsAsync(combined.Token);
            var requests = new QueryRequest?[document.RootElement.GetArrayLength()];
            var results = new QueryBatchItem[requests.Length];
            var index = 0;
            foreach (var entry in document.RootElement.EnumerateArray())
            {
                var operation = ReadOperation(entry);
                try
                {
                    var request = ReadRequest(entry);
                    request.Validate(index, parse.GetValue(expectedSnapshot) is not null);
                    requests[index] = request;
                }
                catch (ContextFailure failure) { results[index] = QueryBatchItem.Failure(operation, failure); }
                index++;
            }

            if (!requests.Any(item => item is not null)) return await Report().WriteAsync(0);

            if (parse.GetValue(overlay) is { } manifest)
            {
                var report = await AnalysisOverlay.RunAsync(input, manifest, timeout, (analysis, files, error) =>
                {
                    if (error is null) return Execute(analysis, files);
                    check = CheckResult.From(analysis) with { OverlayFiles = files, Diagnostics = [] };
                    throw error;
                }, combined.Token);
                return await report.WriteAsync(0);
            }

            var workspace = await FileWorkspace.OpenAsync(input, combined.Token);
            var checkedReport = Execute(workspace.Analysis!, null);
            await workspace.VerifyAsync(combined.Token);
            return await checkedReport.WriteAsync(0);

            QueryBatchReport Execute(WorkspaceAnalysis analysis, IReadOnlyList<string>? files)
            {
                SnapshotGuard.Verify(parse, expectedSnapshot, analysis.Context.SnapshotId);
                check = CheckResult.From(analysis).PageDiagnostics(diagnostics) with { OverlayFiles = files };
                var query = new Lazy<SourceQuery>(() => new SourceQuery(analysis));
                for (var current = 0; current < requests.Length; current++)
                {
                    if (requests[current] is not { } request) continue;
                    try { results[current] = ExecuteQuery(request, analysis, query, requests, results, combined.Token); }
                    catch (ContextFailure failure) { results[current] = QueryBatchItem.Failure(request.Operation, failure); }
                    catch (ArgumentException exception) { results[current] = QueryBatchItem.Failure(request.Operation, new ContextFailure("invalid_argument", exception.Message)); }
                }
                return Report();
            }

            QueryBatchReport Report() => new("ok", check?.Context ?? input.Context, input.Configuration, check?.SnapshotId,
                check?.Diagnostics ?? [], check?.Coverage, check?.Preparation, results, null)
                { DiagnosticPage = check?.DiagnosticPage, OverlayFiles = check?.OverlayFiles };
        }
        catch (OperationCanceledException) when (combined.IsCancellationRequested)
        {
            return token.IsCancellationRequested
                ? await Failure("cancelled", "Query batch cancelled.", 130)
                : await Failure("timeout", "Query batch exceeded --timeout.", 1);
        }
        catch (ContextFailure failure)
        {
            return await QueryBatchReport.Failure(input.Context, input.Configuration,
                AnalysisError.From(failure), check)
                .WriteAsync(failure.Code is "invalid_argument" or "ambiguous_project" ? 2 : 1);
        }
        catch (Exception exception) when (exception is JsonException || exception is ArgumentException and not DecoderFallbackException)
        {
            return await Failure("invalid_argument", exception.Message, 2);
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or PlatformNotSupportedException or DecoderFallbackException)
        {
            return await Failure("context_failed", exception.Message, 1);
        }

        Task<int> Failure(string code, string message, int exit)
            => QueryBatchReport.Failure(input.Context, input.Configuration, new(code, message), check).WriteAsync(exit);
    }

    private static QueryBatchItem ExecuteQuery(QueryRequest request, WorkspaceAnalysis analysis, Lazy<SourceQuery> query,
        QueryRequest?[] requests, QueryBatchItem[] results, CancellationToken token)
    {
        if (request.From is not { } from)
            return QueryBatchItem.From(request.Operation, request.Execute(analysis, query, token));

        var prior = results[from];
        if (prior.Result?.Find is not { } found || found.TotalCount == 0 || found.TotalCount == 1 && found.ReturnedCount == 0)
            throw new ContextFailure("query_dependency_failed", $"Batch entry {from} must return a find declaration.");
        if (found.TotalCount > 1)
            return QueryBatchItem.Failure(request.Operation, new("ambiguous_symbol", $"Batch entry {from} identified several declarations. Narrow the find request."))
                with { Candidates = found.Matches, CandidateCount = found.TotalCount };
        var selected = found.Matches[0];
        var complete = requests[from]!.Project is null ? analysis.IsComplete
            : analysis.Projects.Any(project => project.Project == selected.Project && project.Result?.Availability == "complete");
        if (!complete)
            throw new ContextFailure("analysis_unavailable", $"Batch entry {from} has incomplete coverage. Resolve its project issues before selecting a unique declaration.");
        return QueryBatchItem.From(request.Operation, request.Execute(analysis, query, token, selected));
    }

    private static QueryRequest ReadRequest(JsonElement entry)
    {
        if (entry.ValueKind == JsonValueKind.Object && entry.EnumerateObject().Any(property => property.Value.ValueKind == JsonValueKind.Null))
            throw new ContextFailure("invalid_argument", "Omit optional query fields instead of supplying null.");
        try
        {
            return entry.Deserialize(QueryJson.Default.QueryRequest)
                ?? throw new ContextFailure("invalid_argument", "Expected a query object.");
        }
        catch (Exception exception) when (exception is JsonException or InvalidOperationException)
        {
            throw new ContextFailure("invalid_argument", exception.Message);
        }
    }

    private static string? ReadOperation(JsonElement entry)
    {
        try
        {
            return entry.ValueKind == JsonValueKind.Object && entry.TryGetProperty("operation", out var name) &&
                name.ValueKind == JsonValueKind.String ? name.GetString() : null;
        }
        catch (InvalidOperationException) { return null; }
    }

    private static async Task<JsonDocument> ReadRequestsAsync(CancellationToken token)
    {
        using var stdin = Console.OpenStandardInput();
        var bytes = new byte[InputLimit + 1];
        var length = 0;
        while (length < bytes.Length)
        {
            var read = await stdin.ReadAsync(bytes.AsMemory(length), token).AsTask().WaitAsync(token);
            if (read == 0) break;
            length += read;
        }
        if (length > InputLimit) throw new ContextFailure("invalid_argument", "Query batch input exceeds 256 KiB.");
        try { _ = StrictUtf8.GetCharCount(bytes.AsSpan(0, length)); }
        catch (DecoderFallbackException) { throw new ContextFailure("invalid_argument", "Expected UTF-8 JSON input."); }
        var document = JsonDocument.Parse(bytes.AsMemory(0, length));
        if (document.RootElement.ValueKind == JsonValueKind.Array && document.RootElement.GetArrayLength() is > 0 and <= RequestLimit)
            return document;
        document.Dispose();
        throw new ContextFailure("invalid_argument", "Expected a JSON array with 1 to 64 queries.");
    }
}
