using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;
using System.Text.Json;

namespace Fpas.Workspaces.Analysis;

internal static class SourceGraphAnalysis
{
    internal static WorkspaceAnalysis Analyze(WorkspaceContext context, WorkspaceTextSnapshot texts, ProjectAnalysisCache cache, CancellationToken token)
    {
        var paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var inputs = context.Projects.ToDictionary(project => project.Path, paths);
        var results = new Dictionary<string, ProjectAnalysis>(paths); var updated = new Dictionary<string, ContextProject>(paths);
        var images = new Dictionary<string, MetadataImage>(paths); var active = new HashSet<string>(paths);
        foreach (var image in context.Projects.SelectMany(project => project.Request.ReferenceImages ?? [])) images[image.Path] = image;
        foreach (var reference in context.Projects.SelectMany(project => project.Request.References).Distinct(paths))
        {
            token.ThrowIfCancellationRequested();
            if (!images.ContainsKey(reference) && !context.DependencyOutputs.Any(output => paths.Equals(output.Path, reference) &&
                Path.GetExtension(output.Project).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase)))
                images.Add(reference, context.CaptureReference(reference));
        }
        foreach (var path in context.RequestedProjects) Visit(path);
        var snapshot = context.WithProjects(updated.Values.OrderBy(project => project.Path, StringComparer.Ordinal), texts);
        return new(snapshot, Array.AsReadOnly(results.Values.OrderBy(project => project.Project, StringComparer.Ordinal).ToArray()));

        ProjectAnalysis Visit(string path)
        {
            token.ThrowIfCancellationRequested();
            if (results.TryGetValue(path, out var previous)) return previous;
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
                var request = input.Request with { Sources = Array.AsReadOnly(input.Request.Sources.Select(source =>
                    new CompilationSource(source.Path, texts.Document(source.Path).Text)).ToArray()) };
                if (request.References.Any(reference => !images.ContainsKey(reference))) blocked = true;
                if (blocked) { updated[path] = input with { Request = request }; return results[path] = new(path, input.Configuration, "blocked", null); }
                request = request with { ReferenceImages = Array.AsReadOnly(request.References.Select(reference => images[reference]).ToArray()) };
                updated[path] = input with { Request = request };
                var key = WorkspaceContext.HashText(path + "\n" + input.Configuration + "\n" + request.AssemblyName + "\n" + request.Target + "\n" +
                    request.ValidateDocumentation + "\n" + JsonSerializer.Serialize(request.DiagnosticOptions) + "\n" +
                    string.Join('\n', request.Sources.Select(source => source.Path + "=" + texts.Document(source.Path).ContentHash)) + "\n" +
                    string.Join('\n', request.ReferenceImages.Select(image => image.Path + "=" + image.ContentHash)));
                var required = context.DependencyOutputs.Where(output => paths.Equals(output.Project, path) &&
                    context.Projects.Any(project => project.Request.References.Contains(output.Path, paths))).ToArray();
                var cached = cache.Get(key);
                if (cached is null || required.Length != 0 && cached.Analysis.Success && cached.Output is null)
                {
                    var result = FpasCompiler.Analyze(request, token); MetadataImage? output = null;
                    if (result.Success && required.Length != 0)
                    {
                        var compiled = FpasCompiler.Compile(new(request.Sources, request.AssemblyName, request.References, request.Target,
                            DiagnosticOptions: request.DiagnosticOptions, ReferenceImages: request.ReferenceImages), token);
                        if (!compiled.Success) result = new(false, compiled.Diagnostics);
                        else output = new(required[0].Path, compiled.Program!.AssemblyImage);
                    }
                    cached = new(result, output, ProjectAnalysisCache.Measure(result, output)); cache.Add(key, cached);
                }
                if (cached.Output is { } image) foreach (var output in required) images[output.Path] = image.WithPath(output.Path);
                return results[path] = new(path, input.Configuration, cached.Analysis.Success ? "checked" : "invalid", cached.Analysis);
            }
            finally { active.Remove(path); }
        }
    }
}
