using Fpas.Compiler;
using Fpas.Workspaces.Loading;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Testing;

public sealed class TestOutput
{
    private readonly Dictionary<string, SourceLocation?> methods = new(StringComparer.Ordinal);
    public TestProject Project { get; private set; }

    public TestOutput(TestProject project, CancellationToken token)
    {
        TestDriver.RequireOutput(project);
        var pdb = Path.ChangeExtension(project.Assembly, ".pdb");
        var identity = Identity(project, token);
        Project = project with { OutputIdentity = identity };
        if (!File.Exists(pdb)) return;
        try
        {
            using var image = new PEReader(File.OpenRead(project.Assembly));
            using var symbols = MetadataReaderProvider.FromPortablePdbStream(File.OpenRead(pdb));
            var reader = symbols.GetMetadataReader();
            var id = new BlobContentId(reader.DebugMetadataHeader!.Id);
            if (!image.ReadDebugDirectory().Any(entry => entry.Type == DebugDirectoryEntryType.CodeView &&
                entry.Stamp == id.Stamp && image.ReadCodeViewDebugDirectoryData(entry).Guid == id.Guid))
            { Project = Project with { StaleSources = true }; return; }

            var sources = new Dictionary<DocumentHandle, string>();
            foreach (var handle in reader.Documents)
            {
                token.ThrowIfCancellationRequested();
                var document = reader.GetDocument(handle);
                var path = reader.GetString(document.Name);
                if (!Path.GetExtension(path).Equals(".fpas", StringComparison.OrdinalIgnoreCase)) continue;
                if (!Path.IsPathFullyQualified(path) || !File.Exists(path))
                { Project = Project with { StaleSources = true }; continue; }
                using var source = File.OpenRead(path);
                var algorithm = reader.GetGuid(document.HashAlgorithm);
                var hash = algorithm == new Guid("8829d00f-11b8-4213-878b-770e8597ac16") ? SHA256.HashData(source) :
                    algorithm == new Guid("ff1816ec-aa5e-4d10-87f7-6f4963833460") ? SHA1.HashData(source) : [];
                if (hash.Length != 0 && hash.AsSpan().SequenceEqual(reader.GetBlobBytes(document.Hash))) sources.Add(handle, path);
                else Project = Project with { StaleSources = true };
            }
            if (sources.Count == 0) return;
            var metadata = image.GetMetadataReader();
            foreach (var handle in metadata.MethodDefinitions)
            {
                token.ThrowIfCancellationRequested();
                var method = metadata.GetMethodDefinition(handle);
                var name = TypeName(method.GetDeclaringType()) + "." + metadata.GetString(method.Name);
                var info = reader.GetMethodDebugInformation(handle.ToDebugInformationHandle());
                SourceLocation? location = null;
                foreach (var point in info.GetSequencePoints())
                {
                    if (point.IsHidden || !sources.TryGetValue(point.Document.IsNil ? info.Document : point.Document, out var file)) continue;
                    location = new(file, point.StartLine, point.StartColumn, 0, point.EndLine, point.EndColumn);
                    break;
                }
                if (!methods.TryAdd(name, location)) methods[name] = null;
            }

            string TypeName(TypeDefinitionHandle handle)
            {
                var type = metadata.GetTypeDefinition(handle);
                var name = metadata.GetString(type.Name);
                var parent = type.GetDeclaringType();
                if (!parent.IsNil) return TypeName(parent) + "+" + name;
                return type.Namespace.IsNil ? name : metadata.GetString(type.Namespace) + "." + name;
            }
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or BadImageFormatException or InvalidOperationException)
        { methods.Clear(); Project = Project with { StaleSources = true }; }
    }

    public IReadOnlyList<DiscoveredTest> AttachLocations(IReadOnlyList<DiscoveredTest> tests)
        => tests.Select(test => test with { Location = methods.GetValueOrDefault(test.FullyQualifiedName) }).ToArray();

    public void Verify(CancellationToken token)
    {
        if (Identity(Project, token) != Project.OutputIdentity)
            throw new ContextFailure("test_output_changed", "Test outputs changed during discovery or execution. Refresh the test view.");
    }

    private static string Identity(TestProject project, CancellationToken token)
    {
        var paths = new[] { project.Assembly, Path.ChangeExtension(project.Assembly, ".pdb"),
            Path.ChangeExtension(project.Assembly, ".deps.json"), Path.ChangeExtension(project.Assembly, ".runtimeconfig.json") };
        return Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(string.Join(':', paths.Select(path =>
        {
            token.ThrowIfCancellationRequested();
            if (!File.Exists(path)) return "-";
            using var content = File.OpenRead(path);
            return Convert.ToHexString(SHA256.HashData(content));
        })))));
    }
}
