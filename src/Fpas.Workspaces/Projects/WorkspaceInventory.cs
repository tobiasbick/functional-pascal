// [Documentation](../../../docs/pascal/program-structure/project-commands.md#inspect-the-workspace)
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Solutions;
using System.Text.Json;

namespace Fpas.Workspaces.Projects;

/// <summary>Describes the configuration, role and direct references of a project in a workspace.</summary>
/// <param name="Project">Absolute project path.</param>
/// <param name="Configuration">Mapped project configuration.</param>
/// <param name="Platform">Mapped project platform.</param>
/// <param name="BuildEnabled">Whether the solution configuration builds this project.</param>
/// <param name="Frameworks">Evaluated target frameworks, with roles and references.</param>
/// <param name="Issue">Evaluation failure for this project, if any.</param>
public sealed record WorkspaceProject(string Project, string Configuration, string Platform, bool BuildEnabled,
    IReadOnlyList<WorkspaceFramework> Frameworks, ContextIssue? Issue = null);

/// <summary>Evaluated project information for one target framework.</summary>
/// <param name="Framework">Evaluated framework, or null when the project declares none.</param>
/// <param name="Role">test, executable, library or other.</param>
/// <param name="OutputType">MSBuild output kind, when defined.</param>
/// <param name="References">Direct evaluated ProjectReference paths, including external projects.</param>
public sealed record WorkspaceFramework(string? Framework, string Role, string? OutputType, IReadOnlyList<string> References);

/// <summary>Identifies the known owners of one input source in the evaluated context.</summary>
/// <param name="File">Absolute source path.</param>
/// <param name="Owners">Projects that compile the file in at least one evaluated framework.</param>
public sealed record WorkspaceSourceOwners(string File, IReadOnlyList<string> Owners);

/// <summary>Describes the project-level impact of changing selected sources.</summary>
/// <param name="Complete">Whether all projects and reference edges could be evaluated within the context.</param>
/// <param name="Sources">Input files and their known compile owners.</param>
/// <param name="Dependents">Direct and transitive consumers of the owning projects, excluding the owners themselves.</param>
/// <param name="Tests">Affected projects with at least one evaluated test target.</param>
/// <param name="ExternalReferences">Project reference paths outside the evaluated context.</param>
public sealed record WorkspaceImpact(bool Complete, IReadOnlyList<WorkspaceSourceOwners> Sources, IReadOnlyList<string> Dependents,
    IReadOnlyList<string> Tests, IReadOnlyList<string> ExternalReferences);

/// <summary>Discovers workspace projects using MSBuild evaluation with installed SDKs.</summary>
public static class WorkspaceInventory
{
    /// <summary>Returns supported project entries in the chosen solution configuration, or one standalone project.</summary>
    /// <param name="context">Solution or project path.</param>
    /// <param name="configuration">Requested solution or project configuration.</param>
    /// <param name="token">Cancels evaluation and stops owned processes.</param>
    /// <param name="sourceFile">Optional source path to match against evaluated Compile items.</param>
    /// <returns>Matching project descriptions, including per-project issues when evaluation fails.</returns>
    public static Task<IReadOnlyList<WorkspaceProject>> ReadAsync(string context, string configuration, CancellationToken token, string? sourceFile = null)
        => EvaluateAsync(context, configuration, token, sourceFile is null ? null : Path.GetFullPath(sourceFile));

    /// <summary>Finds source owners and follows evaluated reverse project references within the context.</summary>
    /// <param name="context">Solution or project path.</param>
    /// <param name="configuration">Requested solution or project configuration.</param>
    /// <param name="files">Source paths to match against evaluated Compile items.</param>
    /// <param name="token">Cancels evaluation and graph traversal.</param>
    /// <returns>Known affected projects and evaluation issues, together with the impact summary.</returns>
    public static async Task<(IReadOnlyList<WorkspaceProject> Projects, WorkspaceImpact Impact)> ReadAffectedAsync(string context,
        string configuration, IReadOnlyList<string> files, CancellationToken token)
    {
        var paths = WorkspacePaths.Comparer;
        var owners = files.Select(Path.GetFullPath).Distinct(paths).ToDictionary(file => file, _ => new HashSet<string>(paths), paths);
        var projects = await EvaluateAsync(context, configuration, token, affectedOwners: owners);
        var ownersSet = owners.Values.SelectMany(value => value).ToHashSet(paths);
        var affected = new HashSet<string>(ownersSet, paths);
        var edges = projects.SelectMany(project => project.Frameworks.SelectMany(framework => framework.References)
            .Distinct(paths).Select(reference => (Reference: reference, Consumer: project.Project))).ToArray();
        var consumers = edges.ToLookup(edge => edge.Reference, edge => edge.Consumer, paths);
        var pending = new Queue<string>(ownersSet);
        while (pending.TryDequeue(out var reference))
        {
            token.ThrowIfCancellationRequested();
            foreach (var consumer in consumers[reference])
                if (affected.Add(consumer)) pending.Enqueue(consumer);
        }
        var known = projects.Select(project => project.Project).ToHashSet(paths);
        var external = edges.Select(edge => edge.Reference).Where(reference => !known.Contains(reference)).Distinct(paths).Order(StringComparer.Ordinal).ToArray();
        var tests = projects.Where(project => affected.Contains(project.Project) && project.Frameworks.Any(framework => framework.Role == "test"))
            .Select(project => project.Project).ToArray();
        var impact = new WorkspaceImpact(external.Length == 0 && projects.All(project => project.Issue is null),
            owners.OrderBy(entry => entry.Key, StringComparer.Ordinal).Select(entry => new WorkspaceSourceOwners(entry.Key, entry.Value.Order(StringComparer.Ordinal).ToArray())).ToArray(),
            affected.Where(project => !ownersSet.Contains(project)).Order(StringComparer.Ordinal).ToArray(), tests, external);
        return (projects.Where(project => affected.Contains(project.Project) || project.Issue is not null).ToArray(), impact);
    }

    private static async Task<IReadOnlyList<WorkspaceProject>> EvaluateAsync(string context, string configuration, CancellationToken token,
        string? sourceFile = null, Dictionary<string, HashSet<string>>? affectedOwners = null)
    {
        context = ProjectDriver.ValidateContext(context);
        var includeSources = sourceFile is not null || affectedOwners is not null;
        var scope = await ProjectDriver.Scope(context, configuration, token, includeExcluded: true);
        var scratch = Directory.CreateTempSubdirectory("fpas-workspace-").FullName;
        try
        {
            var environment = await WorkspacePreparation.InstalledSdkEnvironment(context, scratch, token);
            var projects = new List<WorkspaceProject>();
            foreach (var project in scope.Projects.OrderBy(project => project.Path, StringComparer.Ordinal))
            {
                var frameworks = new List<WorkspaceFramework>();
                ContextIssue? issue = null;
                try
                {
                    using var evaluation = await Evaluate(project, scope, environment, null, includeSources, token);
                    var properties = evaluation.RootElement.GetProperty("Properties");
                    var targets = Property(properties, "TargetFrameworks")?.Split(';', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
                    if (targets is { Length: > 0 })
                    {
                        foreach (var framework in targets.Distinct(StringComparer.Ordinal))
                        {
                            using var inner = await Evaluate(project, scope, environment, framework, includeSources, token);
                            AddIfMatching(inner.RootElement);
                        }
                    }
                    else AddIfMatching(evaluation.RootElement);
                }
                catch (ContextFailure error) { issue = ContextIssue.From(project.Path, error); }
                catch (Exception error) when (error is IOException or UnauthorizedAccessException or JsonException or KeyNotFoundException or InvalidOperationException)
                { issue = new(project.Path, "evaluation_failed", error.Message); }
                if (sourceFile is null || frameworks.Count != 0 || issue is not null)
                    projects.Add(new(project.Path, project.Configuration, project.Platform, project.Build, frameworks.ToArray(), issue));

                void AddIfMatching(JsonElement evaluation)
                {
                    if (affectedOwners is not null)
                        foreach (var item in evaluation.GetProperty("Items").GetProperty("Compile").EnumerateArray())
                            if (affectedOwners.TryGetValue(item.GetProperty("FullPath").GetString()!, out var matching)) matching.Add(project.Path);
                    if (sourceFile is null || evaluation.GetProperty("Items").GetProperty("Compile").EnumerateArray()
                        .Any(item => WorkspacePaths.Comparer.Equals(sourceFile, item.GetProperty("FullPath").GetString())))
                        frameworks.Add(Describe(evaluation));
                }
            }
            return projects;
        }
        finally { await WorkspaceLoader.RemoveScratch(scratch); }
    }

    private static async Task<JsonDocument> Evaluate(ProjectSelection project, SolutionScope scope, IReadOnlyDictionary<string, string> environment,
        string? framework, bool includeSources, CancellationToken token)
    {
        if (!File.Exists(project.Path))
            throw new ContextFailure("evaluation_failed", "Project file is missing.", inputPath: project.Path);
        var arguments = ProjectDriver.EvaluationArguments(project, scope);
        arguments.AddRange(["-getProperty:TargetFramework,TargetFrameworks,IsTestProject,OutputType",
            includeSources ? "-getItem:ProjectReference,Compile" : "-getItem:ProjectReference"]);
        if (framework is not null) arguments.Add("-p:TargetFramework=" + MsBuildSyntax.Escape(framework));
        var result = await BuildProcess.Run(Path.GetDirectoryName(project.Path)!, arguments, token, environment: environment);
        if (result.ExitCode != 0 || result.Truncated)
            throw new ContextFailure("evaluation_failed", "Cannot evaluate project metadata. " + result.Output + result.Error, result.Truncated, inputPath: project.Path);
        return JsonDocument.Parse(result.Output);
    }

    private static WorkspaceFramework Describe(JsonElement evaluation)
    {
        var properties = evaluation.GetProperty("Properties");
        var output = Property(properties, "OutputType");
        var role = string.Equals(Property(properties, "IsTestProject"), "true", StringComparison.OrdinalIgnoreCase) ? "test" : output?.ToLowerInvariant() switch
        {
            "exe" or "winexe" => "executable",
            "library" => "library",
            _ => "other"
        };
        var references = evaluation.GetProperty("Items").GetProperty("ProjectReference").EnumerateArray()
            .Select(reference => reference.GetProperty("FullPath").GetString()!).Distinct(WorkspacePaths.Comparer).Order(StringComparer.Ordinal).ToArray();
        return new(Property(properties, "TargetFramework"), role, output, references);
    }

    private static string? Property(JsonElement properties, string name)
        => properties.GetProperty(name).GetString() is { Length: > 0 } value ? value : null;
}
