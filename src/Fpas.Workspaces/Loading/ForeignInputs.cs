using System.Reflection.Metadata;
using System.Security.Cryptography;
using System.Reflection.PortableExecutable;

namespace Fpas.Workspaces.Loading;

internal static class ForeignInputs
{
    internal static void Capture(PreparationOutput output, string[] sources, Dictionary<string, string> hashes, List<ContextIssue> issues, CancellationToken token)
    {
        var path = Path.ChangeExtension(output.Path, ".pdb");
        try
        {
            using var stream = File.OpenRead(path);
            using var provider = MetadataReaderProvider.FromPortablePdbStream(stream);
            var reader = provider.GetMetadataReader();
            using var assembly = File.OpenRead(output.Path); using var image = new PEReader(assembly);
            var identity = new BlobContentId((reader.DebugMetadataHeader ?? throw new BadImageFormatException("Portable symbol header is missing.")).Id);
            if (!image.ReadDebugDirectory().Any(entry => entry.Type == DebugDirectoryEntryType.CodeView && entry.Stamp == identity.Stamp &&
                image.ReadCodeViewDebugDirectoryData(entry).Guid == identity.Guid))
                throw new InvalidOperationException("Portable symbols do not match the referenced assembly.");
            var documents = new HashSet<string>(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
            hashes[path] = WorkspaceContext.HashFile(path);
            foreach (var handle in reader.Documents)
            {
                token.ThrowIfCancellationRequested();
                var document = reader.GetDocument(handle); var source = reader.GetString(document.Name);
                if (!Path.IsPathFullyQualified(source) || !File.Exists(source)) throw new InvalidOperationException("Source checksums cannot be verified: " + source);
                var algorithm = reader.GetGuid(document.HashAlgorithm); using var content = File.OpenRead(source);
                var sha256 = SHA256.HashData(content); content.Position = 0;
                var actual = algorithm == new Guid("8829d00f-11b8-4213-878b-770e8597ac16") ? sha256 :
                    algorithm == new Guid("ff1816ec-aa5e-4d10-87f7-6f4963833460") ? SHA1.HashData(content) : [];
                hashes[source] = Convert.ToHexString(sha256); documents.Add(source);
                if (actual.Length == 0 || !actual.AsSpan().SequenceEqual(reader.GetBlobBytes(document.Hash)))
                    throw new InvalidOperationException("Foreign sources differ from their existing metadata: " + source);
            }
            if (sources.Any(source => !documents.Contains(source))) throw new InvalidOperationException("The foreign source list differs from its existing metadata.");
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException or BadImageFormatException or InvalidOperationException)
        { issues.Add(new(output.Project, "foreign_metadata_stale", "Build this foreign project explicitly with portable symbols before analysis. " + error.Message)); }
    }
}
