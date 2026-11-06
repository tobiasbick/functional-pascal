using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Snapshots;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Loading;

public sealed record ContextIssue(string Project, string Code, string Message, bool OutputTruncated = false);
public sealed record ContextProject(string Path, string Configuration, string Id, AnalysisRequest Request, IReadOnlyList<string> Dependencies);
public sealed record PreparationOutput(string Project, string Path);

public sealed class WorkspaceContext
{
    public string Path { get; }
    public string Configuration { get; }
    public string SnapshotId { get; }
    public bool RestorePerformed { get; }
    public ContextPreparation Preparation { get; }
    public bool DependencyBuildsAllowed => Preparation == ContextPreparation.BuildDependencies;
    public IReadOnlyList<ContextProject> Projects { get; }
    public IReadOnlyList<ContextIssue> Issues { get; }
    public IReadOnlyList<string> RequestedProjects { get; }
    public IReadOnlyList<PreparationOutput> DependencyOutputs { get; }
    public WorkspaceTextSnapshot Texts { get; }
    private readonly IReadOnlyDictionary<string, string> inputs;

    internal WorkspaceContext(string path, string configuration, bool restored, IEnumerable<ContextProject> projects,
        IEnumerable<ContextIssue> issues, IEnumerable<string> requestedProjects, IEnumerable<PreparationOutput> outputs, IReadOnlyDictionary<string, string> inputs,
        ContextPreparation preparation = ContextPreparation.BuildDependencies, WorkspaceTextSnapshot? texts = null)
    {
        Path = path; Configuration = configuration; RestorePerformed = restored; Preparation = preparation;
        Projects = Array.AsReadOnly(projects.ToArray()); Issues = Array.AsReadOnly(issues.ToArray()); this.inputs = inputs;
        Texts = texts ?? WorkspaceTextSnapshot.Capture(Projects.SelectMany(project => project.Request.Sources));
        RequestedProjects = Array.AsReadOnly(requestedProjects.Order(StringComparer.Ordinal).ToArray());
        DependencyOutputs = Array.AsReadOnly(outputs.Distinct().OrderBy(output => output.Project, StringComparer.Ordinal).ThenBy(output => output.Path, StringComparer.Ordinal).ToArray());
        var semantics = Projects.Select(project => project.Path + "\n" + project.Configuration + "\n" + project.Request.AssemblyName + "\n" + project.Request.Target + "\n" +
            project.Request.ValidateDocumentation + "\n" + project.Request.DiagnosticOptions?.TreatWarningsAsErrors + "\n" +
            string.Join(',', project.Request.DiagnosticOptions?.WarningsAsErrors ?? []) + "\n" +
            string.Join(',', project.Request.DiagnosticOptions?.WarningsNotAsErrors ?? []) + "\n" + string.Join(',', project.Request.DiagnosticOptions?.NoWarn ?? []));
        SnapshotId = HashText(string.Join('\n', inputs.OrderBy(item => item.Key, StringComparer.Ordinal).Select(item => item.Key + "=" + item.Value)) + "\n" +
            configuration + "\n" + string.Join('\n', semantics) + "\n" + string.Join('\n', Issues.Select(issue => issue.Project + ":" + issue.Code)));
    }

    public async Task VerifyInputsAsync(CancellationToken cancellationToken = default)
    {
        VerifyFiles(cancellationToken);
        var current = await WorkspaceLoader.LoadAsync(new(Path, Configuration, NoRestore: true, Preparation), cancellationToken);
        if (current.SnapshotId != SnapshotId) throw new ContextFailure("inputs_changed", "Evaluated project inputs changed after context capture.");
    }

    internal void VerifyFiles(CancellationToken cancellationToken = default, IEnumerable<string>? ignoredSources = null)
    {
        var ignored = IgnoredSourcePaths(ignoredSources ?? []);
        foreach (var (path, expected) in inputs)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (ignored.Contains(path)) continue;
            if (!File.Exists(path) || HashFile(path) != expected) throw new ContextFailure("inputs_changed", "An analysis input changed: " + path);
        }
    }

    internal static string HashFile(string path) { using var stream = File.OpenRead(path); return Convert.ToHexString(SHA256.HashData(stream)); }
    internal static string HashText(string text) => Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(text)));

    internal MetadataImage CaptureReference(string path)
    {
        var image = new MetadataImage(path, File.ReadAllBytes(path));
        if (inputs.TryGetValue(path, out var expected) && image.ContentHash != expected)
            throw new ContextFailure("inputs_changed", "Reference changed during capture: " + path);
        return image;
    }

    internal WorkspaceContext WithProjects(IEnumerable<ContextProject> projects, WorkspaceTextSnapshot texts)
    {
        var captured = projects.ToArray();
        var hashes = new Dictionary<string, string>(inputs, OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
        foreach (var document in texts.Documents) hashes[document.Path] = document.ContentHash;
        foreach (var image in captured.SelectMany(project => project.Request.ReferenceImages ?? [])) hashes[image.Path] = image.ContentHash;
        return new(Path, Configuration, false, captured, Issues, RequestedProjects, DependencyOutputs, hashes, Preparation, texts);
    }

    internal string InputIdentity(IEnumerable<string> ignoredSources)
    {
        var ignored = IgnoredSourcePaths(ignoredSources);
        return HashText(string.Join('\n', inputs.OrderBy(pair => pair.Key, StringComparer.Ordinal).Select(pair => pair.Key + "=" + (ignored.Contains(pair.Key) ? "open" : pair.Value))) +
            "\n" + Configuration + "\n" + System.Text.Json.JsonSerializer.Serialize(Projects.Select(project => new
            { project.Path, project.Configuration, project.Request.AssemblyName, project.Request.Target, project.Request.ValidateDocumentation, project.Request.DiagnosticOptions, project.Dependencies })) +
            "\n" + string.Join('\n', Issues.Select(issue => issue.Project + ":" + issue.Code)));
    }

    private HashSet<string> IgnoredSourcePaths(IEnumerable<string> paths)
    {
        var comparer = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var owned = Texts.Documents.Select(document => document.Path).ToHashSet(comparer);
        return paths.Where(owned.Contains).ToHashSet(comparer);
    }
}
