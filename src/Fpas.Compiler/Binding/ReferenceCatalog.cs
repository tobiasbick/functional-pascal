using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog : IDisposable
{
    private readonly List<ReferenceAssembly> assemblies = [];
    private readonly Dictionary<string, List<ImportedType>> types = new(StringComparer.Ordinal);
    internal AssemblyIdentity Runtime { get; private set; } = null!;

    internal static ReferenceCatalog Open(IReadOnlyList<string> paths, SourceText source)
    {
        var catalog = new ReferenceCatalog();
        try
        {
            var names = new HashSet<string>(StringComparer.Ordinal);
            foreach (var path in paths.Select(Path.GetFullPath).Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal))
            {
                ReferenceAssembly assembly;
                try { assembly = new(path); }
                catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or BadImageFormatException)
                {
                    throw source.Error("FP4001", $"Cannot read reference '{path}': {exception.Message}", 0);
                }
                catalog.assemblies.Add(assembly);
                if (!names.Add(assembly.Identity.Name))
                    throw source.Error("FP4002", $"Multiple references declare assembly '{assembly.Identity.Name}'.", 0);
                foreach (var handle in assembly.Reader.TypeDefinitions)
                {
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
                ?? throw source.Error("FP4001", "Supply the net10.0 System.Runtime reference assembly.", 0);
            if (catalog.Runtime.Version.Major != 10)
                throw source.Error("FP4001", "This compiler currently requires net10.0 reference assemblies.", 0);
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
