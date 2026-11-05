using Fpas.Compiler;
using Fpas.Cli.Diagnostics;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed record AnalysisDiagnostic : CompilerDiagnostic
{
    public string Project { get; }
    internal AnalysisDiagnostic(string project, CompilerDiagnostic diagnostic) : base(diagnostic) => Project = project;
}
internal sealed record AnalysisError(string Code, string Message, bool OutputTruncated = false, IReadOnlyList<string>? Candidates = null);
internal sealed record AnalysisCoverage(bool Complete, IReadOnlyList<string> Requested, IReadOnlyList<string> Analyzed, IReadOnlyList<string> Blocked, IReadOnlyList<string>? Searched = null);
internal sealed record Preparation(bool RestorePerformed, bool DependencyBuildsAllowed, IReadOnlyList<PreparationOutput> DependencyOutputs, IReadOnlyList<ContextIssue> Issues);
internal sealed record CheckedProject(string Project, string? Configuration, string State, IReadOnlyList<AnalysisDiagnostic> Diagnostics);
internal sealed record CheckResult(bool Success, string Status, string? Context, string Configuration, string? SnapshotId,
    IReadOnlyList<CheckedProject> Projects, IReadOnlyList<AnalysisDiagnostic> Diagnostics, AnalysisCoverage Coverage, Preparation? Preparation, AnalysisError? Error)
{
    [JsonIgnore] internal IReadOnlyList<CompilationSource> Sources { get; init; } = [];
    internal static CheckResult From(WorkspaceAnalysis analysis)
    {
        var context = analysis.Context;
        var projects = analysis.Projects.Select(project => new CheckedProject(project.Project, project.Configuration, project.State,
            (project.Result?.Diagnostics ?? []).Select(diagnostic => new AnalysisDiagnostic(project.Project, diagnostic)).ToArray())).ToArray();
        return new(analysis.Success, analysis.Success ? "ok" : "failed", context.Path, context.Configuration,
            analysis.IsComplete ? context.SnapshotId : null, projects, projects.SelectMany(project => project.Diagnostics).ToArray(),
            new(analysis.IsComplete, context.RequestedProjects, projects.Where(project => project.State != "blocked").Select(project => project.Project).ToArray(),
                projects.Where(project => project.State == "blocked").Select(project => project.Project).ToArray()),
            new(context.RestorePerformed, true, context.DependencyOutputs, context.Issues),
            analysis.Success ? null : new(context.Issues.Count != 0 ? "context_failed" : "analysis_failed", "The selected context could not be checked successfully."))
            { Sources = context.Projects.SelectMany(project => project.Request.Sources).Distinct().ToArray() };
    }

    internal static CheckResult Failure(string? context, string configuration, string code, string message, bool truncated = false, IReadOnlyList<string>? candidates = null)
        => new(false, code == "cancelled" ? "cancelled" : "failed", context, configuration, null, [], [], new(false, [], [], []), null, new(code, message, truncated, candidates));

    internal void Report(bool json)
    {
        if (json) { Console.WriteLine(JsonSerializer.Serialize(this, CheckJson.Default.CheckResult)); return; }
        DiagnosticRenderer.Write(Diagnostics, Sources);
        foreach (var issue in Preparation?.Issues ?? []) Console.Error.WriteLine($"{issue.Project}: {issue.Code}: {issue.Message}");
        if (Error is not null) Console.Error.WriteLine(Error.Code + ": " + Error.Message);
        Console.Error.WriteLine($"Checked {Coverage.Analyzed.Count}/{Coverage.Requested.Count} FPAS projects; {Coverage.Blocked.Count} blocked. {(Success ? "Success." : "Failed.")}");
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(CheckResult))]
internal partial class CheckJson : JsonSerializerContext;
