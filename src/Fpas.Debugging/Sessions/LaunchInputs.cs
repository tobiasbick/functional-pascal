using Fpas.Debugging.Protocol;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;

namespace Fpas.Debugging.Sessions;

internal static class LaunchInputs
{
    internal static void Validate(string program, string cwd, string[] args)
    {
        if (!Path.IsPathFullyQualified(program) || !File.Exists(program) || !program.EndsWith(".dll", StringComparison.OrdinalIgnoreCase))
            throw new DebugFailure("invalid_argument", "program must be an absolute path to a built DLL.");
        if (!Path.IsPathFullyQualified(cwd) || !Directory.Exists(cwd))
            throw new DebugFailure("invalid_argument", "cwd must be an absolute existing directory.");
        if (args.Length > 1024 || args.Any(argument => argument is null || argument.Contains('\0')))
            throw new DebugFailure("invalid_argument", "args must contain individual strings without NUL characters (maximum 1024).");
        if (!File.Exists(Path.ChangeExtension(program, ".runtimeconfig.json")))
            throw new DebugFailure("missing_runtimeconfig", "Build an executable with its matching runtimeconfig.json and dependencies.");
        var pdb = Path.ChangeExtension(program, ".pdb");
        if (!File.Exists(pdb)) throw new DebugFailure("missing_symbols", "Build the program with portable symbols and keep its matching PDB beside the DLL.");
        try
        {
            using var assembly = File.OpenRead(program); using var pe = new PEReader(assembly);
            using var symbols = File.OpenRead(pdb); using var provider = MetadataReaderProvider.FromPortablePdbStream(symbols);
            var metadata = provider.GetMetadataReader();
            var entry = pe.ReadDebugDirectory().SingleOrDefault(entry => entry.Type == DebugDirectoryEntryType.CodeView);
            if (entry.Type != DebugDirectoryEntryType.CodeView || pe.ReadCodeViewDebugDirectoryData(entry).Guid != new Guid(metadata.DebugMetadataHeader!.Id.AsSpan()[..16]))
                throw new DebugFailure("missing_symbols", "The assembly and portable PDB do not match. Rebuild them together.");
            foreach (var handle in metadata.Documents)
            {
                var document = metadata.GetDocument(handle); var path = metadata.GetString(document.Name);
                if (!Path.IsPathFullyQualified(path) || !File.Exists(path))
                    throw new DebugFailure("missing_source", "The source recorded in the PDB is unavailable: " + path);
                var algorithm = metadata.GetGuid(document.HashAlgorithm);
                var content = File.ReadAllBytes(path);
                var hash = algorithm == new Guid("8829d00f-11b8-4213-878b-770e8597ac16") ? SHA256.HashData(content) :
                    algorithm == new Guid("ff1816ec-aa5e-4d10-87f7-6f4963833460") ? SHA1.HashData(content) : null;
                if (hash is null || !hash.SequenceEqual(metadata.GetBlobBytes(document.Hash)))
                    throw new DebugFailure("missing_source", "Source checksum does not match the PDB: " + path);
            }
        }
        catch (Exception exception) when (exception is BadImageFormatException or InvalidOperationException)
        {
            throw new DebugFailure("missing_symbols", "The DLL/PDB pair is not valid portable debug output.");
        }
    }
}
