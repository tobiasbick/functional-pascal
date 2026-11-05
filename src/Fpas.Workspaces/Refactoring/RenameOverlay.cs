using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Refactoring;

internal sealed class RenameOverlay : IDisposable
{
    private readonly string scratch = Path.Combine(Path.GetTempPath(), "fpas-rename-" + Guid.NewGuid().ToString("N"));
    internal RenameOverlay() => Directory.CreateDirectory(scratch);
    internal WorkspaceAnalysis Analyze(WorkspaceContext context, IReadOnlyDictionary<string, string> texts, CancellationToken token)
    {
        var paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var inputs = context.Projects.ToDictionary(project => project.Path, paths);
        var visited = new Dictionary<string, ProjectAnalysis>(paths); var updated = new Dictionary<string, ContextProject>(paths);
        var references = new Dictionary<string, string>(paths); var outputs = new List<PreparationOutput>(); long bytes = 0;
        foreach (var input in context.Projects.OrderBy(project => project.Path, StringComparer.Ordinal)) Visit(input.Path);
        var overlay = new WorkspaceContext(context.Path, context.Configuration, false, updated.Values, [], context.RequestedProjects, outputs,
            new Dictionary<string, string>(paths));
        return new(overlay, visited.Values.OrderBy(project => project.Project, StringComparer.Ordinal).ToArray());

        void Visit(string path)
        {
            token.ThrowIfCancellationRequested(); if (visited.ContainsKey(path)) return;
            var input = inputs[path];
            foreach (var dependency in input.Dependencies.Where(inputs.ContainsKey)) Visit(dependency);
            var request = input.Request with
            {
                Sources = input.Request.Sources.Select(source => texts.TryGetValue(source.Path, out var text) ? source with { Text = text } : source).ToArray(),
                References = input.Request.References.Select(reference => references.GetValueOrDefault(reference) ?? reference).ToArray()
            };
            var result = FpasCompiler.Analyze(request, token);
            if (!result.Success) throw new ContextFailure("rename_conflict", "Renamed sources fail validation in '" + path + "': " +
                string.Join("; ", result.Diagnostics.Take(10).Select(diagnostic => diagnostic.Code + ": " + diagnostic.Message)));
            updated.Add(path, input with { Request = request }); visited.Add(path, new(path, input.Configuration, "checked", result));
            var required = context.DependencyOutputs.Where(output => paths.Equals(output.Project, path) &&
                context.Projects.Any(project => project.Request.References.Contains(output.Path, paths))).ToArray();
            if (required.Length == 0) return;
            var compiled = FpasCompiler.Compile(new(request.Sources, request.AssemblyName, request.References, request.Target,
                DiagnosticOptions: request.DiagnosticOptions), token);
            if (!compiled.Success) throw new ContextFailure("rename_conflict", "Cannot prepare renamed dependency metadata: " + path);
            if ((bytes += compiled.Program!.AssemblyImage.Length) > 67_108_864) throw new ContextFailure("rename_limit", "Temporary dependency metadata exceeds 64 MiB.");
            var assembly = Path.Combine(scratch, updated.Count.ToString(System.Globalization.CultureInfo.InvariantCulture) + ".dll");
            File.WriteAllBytes(assembly, compiled.Program.AssemblyImage);
            foreach (var output in required) { references[output.Path] = assembly; outputs.Add(new(path, assembly)); }
        }
    }
    public void Dispose() => Directory.Delete(scratch, recursive: true);
}
