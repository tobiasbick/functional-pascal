// [Documentation](../../../docs/pascal/program-structure/cli.md#lint-a-project-or-solution)
using Fpas.Cli.Diagnostics;
using Fpas.Compiler;
using Fpas.Workspaces.Lint;
using Fpas.Workspaces.Loading;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Cli.Analysis;

internal sealed record LintReport(bool Success, string Status, string? Context, string Configuration, string? SnapshotId,
    IReadOnlyList<LintFinding> Findings, IReadOnlyList<AnalysisDiagnostic> Diagnostics,
    IReadOnlyList<ContextIssue> PreparationIssues, AnalysisError? Error)
{
    /// <summary>Gets the lint report contract version.</summary>
    public int SchemaVersion => 1;

    /// <summary>Gets baseline application or publication details when requested.</summary>
    public LintBaselineSummary? Baseline { get; init; }

    internal static LintReport From(CheckResult check, IReadOnlyList<LintFinding> findings, bool ready)
        => new(ready && !findings.Any(LintBaseline.IsError), !ready ? "failed" : findings.Any(LintBaseline.IsError) ? "findings" : "ok",
            check.Context, check.Configuration, check.SnapshotId, findings, check.Diagnostics, check.Preparation?.Issues ?? [],
            ready ? null : check.Error ?? new("analysis_failed", "Lint requires complete source analysis."));

    internal static LintReport Failure(string? context, string configuration, AnalysisError error)
        => new(false, error.Code == "cancelled" ? "cancelled" : "failed", context, configuration, null, [], [], [], error);

    internal bool Write(bool json)
    {
        if (json)
        {
            var bytes = JsonSerializer.SerializeToUtf8Bytes(this, LintJson.Default.LintReport);
            var overflow = bytes.Length > 4_194_304;
            if (overflow)
                bytes = JsonSerializer.SerializeToUtf8Bytes(Failure(null, Configuration,
                    new("result_limit", "Lint response exceeds 4 MiB.", OutputTruncated: true)) with
                    { Baseline = Baseline is null ? null : Baseline with { Resolved = [] } }, LintJson.Default.LintReport);
            using var output = Console.OpenStandardOutput();
            output.Write(bytes);
            Console.WriteLine();
            return !overflow;
        }

        DiagnosticRenderer.Write(Diagnostics);
        DiagnosticRenderer.Write(Findings.Select(finding => new CompilerDiagnostic(finding.Rule, finding.Message,
            new(finding.Location.Path, finding.Location.Line, finding.Location.Column, finding.Location.Length, Offset: finding.Location.Offset), finding.Severity)
            { Related = finding.Related }));
        foreach (var issue in PreparationIssues)
            Console.Error.WriteLine(DiagnosticRenderer.Safe($"{issue.Project}: {issue.Code}: {issue.Message}"));
        if (Error is not null) Console.Error.WriteLine(DiagnosticRenderer.Safe($"{Error.Code}: {Error.Message}"));
        else Console.Error.WriteLine($"Lint found {Findings.Count} issue(s).");
        if (Baseline is { } baseline)
        {
            Console.Error.WriteLine(DiagnosticRenderer.Safe($"Baseline {(baseline.Written ? "written" : "applied")}: {baseline.Path}, {baseline.SuppressedCount} finding(s) covered."));
            foreach (var resolved in baseline.Resolved)
                Console.Error.WriteLine(DiagnosticRenderer.Safe($"  resolved: {resolved.Project} {resolved.Rule} {resolved.Key}"));
        }
        return true;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    Converters = [typeof(SeverityConverter), typeof(ApplicabilityConverter)])]
[JsonSerializable(typeof(LintReport))]
internal partial class LintJson : JsonSerializerContext;
