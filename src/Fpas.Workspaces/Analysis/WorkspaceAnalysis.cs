using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Analysis;

/// <summary>Contains one project's analysis or its dependency-blocked state.</summary>
/// <param name="Project">Project path.</param>
/// <param name="Configuration">Evaluated configuration, or null when unavailable.</param>
/// <param name="State">Checked, invalid or blocked state.</param>
/// <param name="Result">Detached compiler facts and diagnostics, or null for blocked inputs.</param>
public sealed record ProjectAnalysis(string Project, string? Configuration, string State, AnalysisResult? Result);

/// <summary>Contains dependency-ordered analysis results for a captured context.</summary>
/// <param name="Context">Captured project inputs and ownership.</param>
/// <param name="Projects">Per-project analysis outcomes.</param>
public sealed record WorkspaceAnalysis(WorkspaceContext Context, IReadOnlyList<ProjectAnalysis> Projects)
{
    /// <summary>Gets whether all requested projects checked successfully without preparation issues.</summary>
    public bool Success => Context.Issues.Count == 0 && Projects.All(project => project.State == "checked");
    /// <summary>Gets whether every selected input and semantic region was checked.</summary>
    public bool IsComplete => Context.Issues.Count == 0 && Projects.All(project => project.Result?.IsComplete == true);
    /// <summary>Gets explicit per-project semantic coverage.</summary>
    public WorkspaceSemanticCoverage SemanticCoverage => new(IsComplete, Array.AsReadOnly(Projects.Select(project => new ProjectSemanticCoverage(
        project.Project, project.Result?.Availability ?? "unavailable", project.Result?.Snapshot?.Coverage.Regions ?? [])).ToArray()));

    /// <summary>Analyzes captured inputs without publishing compilation outputs.</summary>
    /// <param name="context">Evaluated inputs to analyze.</param>
    /// <param name="cancellationToken">Cancels dependency traversal and compiler work.</param>
    /// <returns>Current results, including invalid and blocked projects.</returns>
    public static WorkspaceAnalysis Analyze(WorkspaceContext context, CancellationToken cancellationToken = default)
    {
        var paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var inputs = context.Projects.ToDictionary(project => project.Path, paths);
        var results = new Dictionary<string, ProjectAnalysis>(paths);
        var active = new HashSet<string>(paths);
        foreach (var project in context.RequestedProjects) Visit(project);
        return new(context, Array.AsReadOnly(results.Values.OrderBy(project => project.Project, StringComparer.Ordinal).ToArray()));

        ProjectAnalysis Visit(string path)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (results.TryGetValue(path, out var existing)) return existing;
            if (!active.Add(path)) throw new ContextFailure("context_failed", "Cyclic project analysis dependency: " + path);
            try
            {
                if (!inputs.TryGetValue(path, out var input)) return results[path] = new(path, null, "blocked", null);
                var blocked = false;
                foreach (var dependency in input.Dependencies)
                {
                    if (context.Issues.Any(issue => paths.Equals(issue.Project, dependency))) blocked = true;
                    if (inputs.ContainsKey(dependency) && Visit(dependency).State != "checked") blocked = true;
                }
                if (blocked) return results[path] = new(path, input.Configuration, "blocked", null);
                var result = FpasCompiler.Analyze(input.Request, cancellationToken);
                return results[path] = new(path, input.Configuration, result.Success ? "checked" : "invalid", result);
            }
            finally { active.Remove(path); }
        }
    }
}
