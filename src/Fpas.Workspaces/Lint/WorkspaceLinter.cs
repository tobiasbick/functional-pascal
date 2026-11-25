// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Lint;

/// <summary>A lint finding in an evaluated FPAS source.</summary>
/// <param name="Rule">Stable identifier of the lint rule.</param>
/// <param name="Message">Description of the finding.</param>
/// <param name="Project">Project that owns the finding.</param>
/// <param name="Location">Primary source location.</param>
/// <param name="Related">Additional source locations with explanations.</param>
/// <param name="Key">Stable rule-local identity without source coordinates.</param>
/// <param name="Severity">Error or nonblocking warning.</param>
/// <param name="CycleEdges">Stable dependency edges for a cycle finding.</param>
public sealed record LintFinding(string Rule, string Message, string Project, SourceSpan Location, IReadOnlyList<DiagnosticNote> Related,
    string Key, DiagnosticSeverity Severity = DiagnosticSeverity.Error, IReadOnlyList<string>? CycleEdges = null)
{
    internal static DiagnosticNote Note(string message, SourceSpan span) =>
        new(message, new(span.Path, span.Line, span.Column, span.Length, Offset: span.Offset));

    internal static string Relative(string context, string path) =>
        Path.GetRelativePath(Path.GetDirectoryName(context)!, path).Replace(Path.DirectorySeparatorChar, '/');
}

/// <summary>Controls the size and width thresholds for semantic lint.</summary>
/// <param name="HeldInMind">Maximum declaration group, interface list, or exports learned by one consumer.</param>
/// <param name="MaxDirectoryEntries">Maximum immediate FPAS source entries in a directory.</param>
/// <param name="MaxLines">Maximum physical lines in a production source file.</param>
public sealed record LintLimits(int HeldInMind = 7, int MaxDirectoryEntries = 15, int MaxLines = 2_000);

/// <summary>Runs semantic lint rules over checked FPAS workspaces.</summary>
public static class WorkspaceLinter
{
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    /// <summary>Finds code-structure issues in a successfully analyzed workspace.</summary>
    /// <param name="analysis">Complete, successful workspace analysis.</param>
    /// <param name="limits">Positive size and width thresholds.</param>
    /// <param name="token">Cancels rule evaluation.</param>
    /// <returns>Findings ordered by project, source position and rule.</returns>
    /// <exception cref="ArgumentException">Analysis failed, is incomplete, or a threshold is invalid.</exception>
    public static IReadOnlyList<LintFinding> Analyze(WorkspaceAnalysis analysis, LintLimits limits, CancellationToken token = default)
    {
        if (!analysis.Success || !analysis.IsComplete)
            throw new ArgumentException("Semantic lint requires complete, successful workspace analysis.", nameof(analysis));
        if (limits.HeldInMind < 1 || limits.MaxDirectoryEntries < 1 || limits.MaxLines < 1)
            throw new ArgumentOutOfRangeException(nameof(limits), "Lint thresholds must be positive.");

        var findings = new List<LintFinding>();
        var testProjects = analysis.Context.Projects.Where(project => project.IsTestProject)
            .Select(project => project.Path).ToHashSet(Paths);
        var snapshots = analysis.Projects.ToDictionary(project => project.Project, project => project.Result!.Snapshot!, Paths);
        var bindings = new SourceQuery(analysis);
        var namespaceCycles = new NamespaceCycles(snapshots, bindings);
        var owners = new Dictionary<string, int>(Paths);
        var unused = new Dictionary<string, List<(string Project, SemanticSymbol Alias)>>(Paths);
        foreach (var project in analysis.Projects)
        {
            token.ThrowIfCancellationRequested();
            var snapshot = snapshots[project.Project];
            var references = ArchitectureReferences.BoundUses(snapshot, token).ToArray();
            findings.AddRange(ModuleCycles.Find(project.Project, snapshot, references, token));
            namespaceCycles.Add(project.Project, references, token);
            findings.AddRange(Tangles.Find(project.Project, snapshot, references, limits.HeldInMind, token));
            findings.AddRange(ShallowModules.Find(project.Project, snapshot, limits.HeldInMind, token));
            if (!testProjects.Contains(project.Project))
                findings.AddRange(ShallowModules.FindConsumers(project.Project, references, snapshots, bindings, testProjects, limits.HeldInMind, token));
            findings.AddRange(FieldLeaks.Find(project.Project, snapshot, token));
            findings.AddRange(ExportedSynchronization.Find(project.Project, snapshot, token));
            if (!testProjects.Contains(project.Project))
                findings.AddRange(LonelyProtocols.Find(project.Project, snapshot, token));
            foreach (var document in snapshot.Documents)
                owners[document.Path] = owners.GetValueOrDefault(document.Path) + 1;
            foreach (var alias in ImportUsage.UnusedClrAliases(snapshot, token))
            {
                var path = alias.Declaration!.Path;
                if (!unused.TryGetValue(path, out var aliases)) unused.Add(path, aliases = []);
                aliases.Add((project.Project, alias));
            }
        }

        foreach (var (path, aliases) in unused)
            foreach (var group in aliases.GroupBy(item => item.Alias.Declaration!.Offset))
            {
                token.ThrowIfCancellationRequested();
                if (group.Count() != owners[path]) continue;
                var (project, alias) = group.First();
                findings.Add(new("unused-clr-alias", $"CLR import alias '{alias.Name}' is unused.", project, alias.Declaration!, [],
                    $"{LintFinding.Relative(project, path)}::{alias.Name}"));
            }

        findings.AddRange(SourceLayout.Find(analysis, testProjects, limits, token));
        findings.AddRange(namespaceCycles.Find(token));
        return findings.OrderBy(finding => finding.Project, StringComparer.Ordinal)
            .ThenBy(finding => finding.Location.Path, StringComparer.Ordinal)
            .ThenBy(finding => finding.Location.Offset).ThenBy(finding => finding.Rule, StringComparer.Ordinal).ToArray();
    }
}
