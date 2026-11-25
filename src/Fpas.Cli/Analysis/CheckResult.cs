// [Documentation](../../../docs/pascal/program-structure/cli.md#check-a-project-or-solution)
using Fpas.Compiler;
using Fpas.Cli.Diagnostics;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed record AnalysisDiagnostic : CompilerDiagnostic
{
    /// <summary>Gets the project that produced the diagnostic.</summary>
    public string Project { get; }
    /// <summary>Gets whether the diagnostic was summarized to fit the output budget.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)] public bool DetailsOmitted { get; init; }
    internal AnalysisDiagnostic(string project, CompilerDiagnostic diagnostic) : base(diagnostic) => Project = project;
}
internal sealed record AnalysisError(string Code, string Message, bool OutputTruncated = false, IReadOnlyList<string>? Candidates = null,
    string? Project = null, string? InputPath = null, string? NextAction = null)
{
    /// <summary>Gets the snapshot identity requested by the caller, if it did not match.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] public string? ExpectedSnapshotId { get; init; }
    /// <summary>Gets the snapshot identity captured during analysis, if it did not match.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] public string? ActualSnapshotId { get; init; }

    internal static AnalysisError From(ContextFailure failure)
        => new(failure.Code, failure.Message, failure.OutputTruncated, failure.Candidates, failure.Project, failure.InputPath, failure.NextAction)
        { ExpectedSnapshotId = failure.ExpectedSnapshotId, ActualSnapshotId = failure.ActualSnapshotId };
}
internal sealed record AnalysisCoverage(bool Complete, IReadOnlyList<string> Requested, IReadOnlyList<string> Analyzed, IReadOnlyList<string> Blocked, IReadOnlyList<string>? Searched = null,
    IReadOnlyList<ProjectSemanticCoverage>? Semantics = null);
internal sealed record Preparation(bool RestorePerformed, bool DependencyBuildsAllowed, IReadOnlyList<PreparationOutput> DependencyOutputs, IReadOnlyList<ContextIssue> Issues);
internal sealed record CheckedProject(string Project, string? Configuration, string State, int DiagnosticCount);
internal sealed record CheckResult(bool Success, string Status, string? Context, string Configuration, string? SnapshotId,
    IReadOnlyList<CheckedProject> Projects, IReadOnlyList<AnalysisDiagnostic> Diagnostics, AnalysisCoverage Coverage, Preparation? Preparation, AnalysisError? Error)
{
    /// <summary>Gets the selected diagnostic filter, pagination and counts.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] public DiagnosticPage? DiagnosticPage { get; init; }
    /// <summary>Gets the source paths analyzed from the supplied draft.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] public IReadOnlyList<string>? OverlayFiles { get; init; }
    [JsonIgnore] internal IReadOnlyList<CompilationSource> Sources { get; init; } = [];
    /// <summary>Gets the correction plan and its publication status, if requested.</summary>
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)] public CheckFixes? Fixes { get; init; }
    internal static CheckResult From(WorkspaceAnalysis analysis)
    {
        var context = analysis.Context;
        var projects = analysis.Projects.Select(project => new CheckedProject(project.Project, project.Configuration, project.State,
            project.Result?.Diagnostics.Count ?? 0)).ToArray();
        var diagnostics = analysis.Projects.SelectMany(project => (project.Result?.Diagnostics ?? [])
            .Select(diagnostic => new AnalysisDiagnostic(project.Project, diagnostic)))
            .OrderBy(diagnostic => diagnostic.Project, StringComparer.Ordinal)
            .ThenBy(diagnostic => diagnostic.Location?.Path ?? diagnostic.InputPath, StringComparer.Ordinal)
            .ThenBy(diagnostic => diagnostic.Location?.Offset)
            .ThenBy(diagnostic => diagnostic.Code, StringComparer.Ordinal)
            .ThenBy(diagnostic => diagnostic.Message, StringComparer.Ordinal).ToArray();
        return new(analysis.Success, analysis.Success ? "ok" : "failed", context.Path, context.Configuration,
            context.SnapshotId, projects, diagnostics,
            new(analysis.IsComplete, context.RequestedProjects, projects.Where(project => project.State != "blocked").Select(project => project.Project).ToArray(),
                projects.Where(project => project.State == "blocked").Select(project => project.Project).ToArray(), Semantics: analysis.SemanticCoverage.Projects),
            new(context.RestorePerformed, context.DependencyBuildsAllowed, context.DependencyOutputs, context.Issues),
            analysis.Success ? null : new(context.Issues.Count != 0 ? "context_failed" : "analysis_failed", "The selected context could not be checked successfully."))
            { Sources = context.Projects.SelectMany(project => project.Request.Sources).Distinct().ToArray() };
    }

    internal static CheckResult Failure(string? context, string configuration, string code, string message, bool truncated = false, IReadOnlyList<string>? candidates = null)
        => new(false, code == "cancelled" ? "cancelled" : "failed", context, configuration, null, [], [], new(false, [], [], []), null, new(code, message, truncated, candidates));

    internal CheckResult PageDiagnostics(DiagnosticSelection selection)
    {
        var (diagnostics, page) = selection.Select(Diagnostics);
        return this with { Diagnostics = diagnostics, DiagnosticPage = page };
    }

    internal byte[] JsonBytes() => JsonSerializer.SerializeToUtf8Bytes(this, CheckJson.Default.CheckResult);
    internal bool Report(bool json)
    {
        if (json)
        {
            var bytes = JsonBytes(); var limited = bytes.Length > 4_194_304;
            if (limited) bytes = (Failure(Context, Configuration, "result_limit", "Check response exceeds 4 MiB; publication status is retained.", truncated: true)
                with { Fixes = Fixes is null ? null : Fixes with { Suggestions = [] } }).JsonBytes();
            using var output = Console.OpenStandardOutput(); output.Write(bytes); Console.WriteLine(); return !limited;
        }
        DiagnosticRenderer.Write(Diagnostics, Sources);
        if (DiagnosticPage is { } page && page.ReturnedCount < page.TotalCount)
            Console.Error.WriteLine($"Showing {page.ReturnedCount} of {page.TotalCount} diagnostics. {page.MatchedCount} match the filters.");
        foreach (var issue in Preparation?.Issues ?? []) Console.Error.WriteLine($"{issue.Project}: {issue.Code}: {issue.Message}");
        if (Error is not null) Console.Error.WriteLine(Error.Code + ": " + Error.Message);
        Fixes?.Write();
        Console.Error.WriteLine($"Checked {Coverage.Analyzed.Count}/{Coverage.Requested.Count} FPAS projects; {Coverage.Blocked.Count} blocked. {(Success ? "Success." : "Failed.")}");
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(CheckResult))]
[JsonSerializable(typeof(AnalysisDiagnostic))]
internal partial class CheckJson : JsonSerializerContext;
