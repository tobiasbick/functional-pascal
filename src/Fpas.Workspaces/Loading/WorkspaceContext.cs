using Fpas.Compiler;
using Fpas.Compiler.Analysis;
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
    public IReadOnlyList<ContextProject> Projects { get; }
    public IReadOnlyList<ContextIssue> Issues { get; }
    public IReadOnlyList<string> RequestedProjects { get; }
    public IReadOnlyList<PreparationOutput> DependencyOutputs { get; }
    private readonly IReadOnlyDictionary<string, string> inputs;

    internal WorkspaceContext(string path, string configuration, bool restored, IEnumerable<ContextProject> projects,
        IEnumerable<ContextIssue> issues, IEnumerable<string> requestedProjects, IEnumerable<PreparationOutput> outputs, IReadOnlyDictionary<string, string> inputs)
    {
        Path = path; Configuration = configuration; RestorePerformed = restored;
        Projects = Array.AsReadOnly(projects.ToArray()); Issues = Array.AsReadOnly(issues.ToArray()); this.inputs = inputs;
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
        var current = await WorkspaceLoader.LoadAsync(new(Path, Configuration, NoRestore: true), cancellationToken);
        if (current.SnapshotId != SnapshotId) throw new ContextFailure("inputs_changed", "Evaluated project inputs changed after context capture.");
    }

    internal void VerifyFiles(CancellationToken cancellationToken = default)
    {
        foreach (var (path, expected) in inputs)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (!File.Exists(path) || HashFile(path) != expected) throw new ContextFailure("inputs_changed", "An analysis input changed: " + path);
        }
    }

    internal static string HashFile(string path) { using var stream = File.OpenRead(path); return Convert.ToHexString(SHA256.HashData(stream)); }
    internal static string HashText(string text) => Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(text)));
}
