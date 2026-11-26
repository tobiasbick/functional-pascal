// [Documentation](../../../docs/pascal/tools/api-documentation.md#generate-checked-fpas-usage)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Documentation;

internal sealed class SourceUsage(IReadOnlyList<ContextProject> projects, IReadOnlyDictionary<string, SemanticSnapshot> snapshots,
    MetadataCatalog metadata, DocumentationSearch search, CancellationToken token)
{
    private readonly Dictionary<string, (ProjectImage? Image, string? Reason)> images = new(WorkspacePaths.Comparer);

    internal ApiUsage Create(string project, SemanticSymbol symbol)
    {
        if (symbol.Metadata is not { } identity || BoundDocumentation.Id(identity) is not { } id || id[0] is not ('M' or 'P' or 'F'))
            return Unavailable("Usage requires a callable declaration, property or field.");
        if (id[0] == 'M' && search.Access is not null)
            return Unavailable("--access requires a property or field.");

        if (!images.TryGetValue(project, out var input))
        {
            input = Capture(project);
            images.Add(project, input);
        }
        if (input.Image is not { } image)
            return Unavailable(input.Reason!);

        var matches = metadata.Search(project, image.Metadata.Path, search with { Name = id }, token,
            image.Metadata, image.Context, image.Containers).Take(2).ToArray();
        return matches.Length == 1 ? matches[0].Read().Usage! :
            Unavailable("The selected declaration does not identify a unique public member in the compiled FPAS project.");
    }

    private (ProjectImage? Image, string? Reason) Capture(string project)
    {
        var request = projects.Single(item => WorkspacePaths.Comparer.Equals(item.Path, project)).Request;
        var result = FpasCompiler.Compile(new(request.Sources, request.AssemblyName, request.References,
            request.Target, DiagnosticOptions: request.DiagnosticOptions, ReferenceImages: request.ReferenceImages), token);
        if (result.Program is not { } program)
        {
            var diagnostic = result.Diagnostics.FirstOrDefault(item => item.Severity == DiagnosticSeverity.Error);
            return (null, diagnostic is null ? "The owning FPAS project could not be compiled." :
                "The owning FPAS project could not be compiled. " + diagnostic.Code + ": " + diagnostic.Message);
        }

        var image = new MetadataImage(project + ".usage.dll", program.AssemblyImage);
        var context = request with
        {
            References = [.. request.References, image.Path],
            ReferenceImages = [.. request.ReferenceImages ?? [], image]
        };
        var containers = snapshots[project].Symbols.Where(symbol => symbol is { Kind: "container", Declaration: not null })
            .Select(symbol => symbol.Name).ToHashSet(StringComparer.Ordinal);
        return (new(image, context, containers), null);
    }

    private static ApiUsage Unavailable(string reason) => new("unavailable", null, [], reason);

    private sealed record ProjectImage(MetadataImage Metadata, AnalysisRequest Context, IReadOnlySet<string> Containers);
}
