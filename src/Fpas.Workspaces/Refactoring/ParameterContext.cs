// [Documentation](../../../docs/pascal/tools/refactoring.md#public-signatures-across-projects)
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static void VerifyPublicParameterContext(Draft draft, IReadOnlyList<RefactorTarget> owners)
    {
        if (!draft.ClosedContext)
            throw Refuse("Confirm a context containing all consumers with --closed-context (closedContext in the language server).");
        var context = draft.Analysis.Context;
        var projects = owners.Where(owner => owner.PublicApi).Select(owner => owner.Project).ToHashSet(Paths);
        foreach (var (project, dependencies) in context.ForeignDependencies.OrderBy(pair => pair.Key, StringComparer.Ordinal))
            if (dependencies.Any(projects.Contains))
                throw Refuse("An affected assembly has a non-editable project consumer: " + project);

        var assemblies = context.Projects.Where(project => projects.Contains(project.Path))
            .Select(project => project.Request.AssemblyName).ToHashSet(StringComparer.OrdinalIgnoreCase);
        var captured = context.Projects.SelectMany(project => project.Request.ReferenceImages ?? [])
            .DistinctBy(image => image.Path, Paths).ToDictionary(image => image.Path, Paths);
        foreach (var path in draft.Before.Query.Boundaries())
        {
            draft.Token.ThrowIfCancellationRequested();
            using var image = (captured.GetValueOrDefault(path) ?? context.CaptureReference(path)).OpenRead();
            using var reader = new PEReader(image);
            var metadata = reader.GetMetadataReader();
            if (metadata.IsAssembly && assemblies.Contains(metadata.GetString(metadata.GetAssemblyDefinition().Name)) ||
                metadata.AssemblyReferences.Any(reference => assemblies.Contains(metadata.GetString(metadata.GetAssemblyReference(reference).Name))))
                throw Refuse("An affected assembly has a non-editable binary boundary: " + path);
        }
    }
}
