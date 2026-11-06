using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Analysis;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog : IDisposable
{
    private readonly List<ReferenceAssembly> assemblies = [];
    private readonly Dictionary<string, List<ImportedType>> types = new(StringComparer.Ordinal);
    internal AssemblyIdentity Runtime { get; private set; } = null!;
    internal IEnumerable<KeyValuePair<string, string>> InputFingerprints => assemblies.Select(assembly =>
        new KeyValuePair<string, string>(assembly.Path, assembly.ContentHash ?? throw new InvalidOperationException("Reference fingerprints were not captured.")));

    internal static ReferenceCatalog Open(IReadOnlyList<string> paths, SourceText source, IReadOnlyList<MetadataImage>? images = null, bool captureFingerprints = false)
    {
        var catalog = new ReferenceCatalog();
        try
        {
            var comparer = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
            var captured = (images ?? []).ToDictionary(image => image.Path, comparer);
            var requested = paths.Select(Path.GetFullPath).ToHashSet(comparer);
            if (captured.Keys.Any(path => !requested.Contains(path))) throw new ArgumentException("A metadata image must identify a requested reference.");
            var names = new HashSet<string>(StringComparer.Ordinal);
            foreach (var path in paths.Select(Path.GetFullPath).Distinct(comparer).Order(StringComparer.Ordinal))
            {
                source.CheckCancellation();
                ReferenceAssembly assembly;
                try { assembly = new(path, captureFingerprints, captured.GetValueOrDefault(path)); }
                catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or BadImageFormatException)
                {
                    throw new CompilationFailure(new("FP4001", $"Cannot read reference '{path}': {exception.Message}", null) { InputPath = path });
                }
                catalog.assemblies.Add(assembly);
                if (!names.Add(assembly.Identity.Name))
                    throw new CompilationFailure(new("FP4002", $"Multiple references declare assembly '{assembly.Identity.Name}'.", null));
                foreach (var handle in assembly.Reader.TypeDefinitions)
                {
                    source.CheckCancellation();
                    var type = assembly.Reader.GetTypeDefinition(handle);
                    var ns = assembly.Reader.GetString(type.Namespace);
                    var name = assembly.Reader.GetString(type.Name);
                    var imported = new ImportedType(assembly, handle, ns, name);
                    if (!imported.IsPublic) continue;
                    var fullName = imported.FullName;
                    if (!catalog.types.TryGetValue(fullName, out var candidates))
                        catalog.types.Add(fullName, candidates = []);
                    candidates.Add(imported);
                }
            }
            catalog.Runtime = catalog.assemblies.SingleOrDefault(a => a.Identity.Name == "System.Runtime")?.Identity
                ?? throw new CompilationFailure(new("FP4001", "Supply the net10.0 System.Runtime reference assembly.", null));
            if (catalog.Runtime.Version.Major != 10)
                throw new CompilationFailure(new("FP4001", "This compiler currently requires net10.0 reference assemblies.", null));
            return catalog;
        }
        catch
        {
            catalog.Dispose();
            throw;
        }
    }

    public void Dispose()
    {
        foreach (var assembly in assemblies)
            assembly.Dispose();
    }
}
