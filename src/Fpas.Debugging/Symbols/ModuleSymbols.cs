using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Symbols;

internal sealed partial class ModuleSymbols : MetadataImage
{
    private readonly Stream symbolStream;
    private readonly MetadataReaderProvider symbols;
    internal MetadataReader Pdb { get; }
    internal bool ContainsSource(string source) => Pdb.Documents.Any(handle => string.Equals(System.IO.Path.GetFullPath(DocumentPath(handle)),
        System.IO.Path.GetFullPath(source), OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal));

    internal ModuleSymbols(string path) : base(path)
    {
        try
        {
            var pdbPath = System.IO.Path.ChangeExtension(Path, ".pdb");
            if (!File.Exists(pdbPath)) throw new DebugException("missing_symbols", "A matching portable PDB is required beside the target assembly.");
            symbolStream = new FileStream(pdbPath, FileMode.Open, FileAccess.Read, FileShare.Read | FileShare.Delete);
            try
            {
                symbols = MetadataReaderProvider.FromPortablePdbStream(symbolStream);
                Pdb = symbols.GetMetadataReader();
                var entry = Image.ReadDebugDirectory().Single(item => item.Type == DebugDirectoryEntryType.CodeView);
                var codeView = Image.ReadCodeViewDebugDirectoryData(entry);
                var id = new BlobContentId(Pdb.DebugMetadataHeader!.Id);
                if (codeView.Guid != id.Guid || entry.Stamp != id.Stamp) throw new DebugException("symbols_mismatch", "The portable PDB does not match its assembly.");
            }
            catch { symbolStream.Dispose(); throw; }
        }
        catch { symbols?.Dispose(); symbolStream?.Dispose(); base.Dispose(); throw; }
    }

    internal void ValidateSources()
    {
        foreach (var handle in Pdb.Documents)
        {
            var status = SourceStatus(handle);
            if (status != "available") throw new DebugException(status, "Source recorded in the portable PDB is missing or changed: " + DocumentPath(handle));
        }
    }

    internal void ValidateSource(string source)
    {
        foreach (var handle in Pdb.Documents.Where(handle => string.Equals(System.IO.Path.GetFullPath(DocumentPath(handle)), System.IO.Path.GetFullPath(source),
            OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal)))
        {
            var status = SourceStatus(handle);
            if (status != "available") throw new DebugException(status, "The requested source is missing or changed: " + source);
        }
    }

    private string DocumentPath(DocumentHandle handle) => Pdb.GetString(Pdb.GetDocument(handle).Name);

    private string SourceStatus(DocumentHandle handle)
    {
        var document = Pdb.GetDocument(handle);
        var path = DocumentPath(handle);
        if (!System.IO.Path.IsPathFullyQualified(path) || !File.Exists(path)) return "missing_source";
        var algorithm = Pdb.GetGuid(document.HashAlgorithm);
        using var stream = File.OpenRead(path);
        var hash = algorithm == new Guid("8829d00f-11b8-4213-878b-770e8597ac16") ? SHA256.HashData(stream)
            : algorithm == new Guid("ff1816ec-aa5e-4d10-87f7-6f4963833460") ? SHA1.HashData(stream) : null;
        return hash is null ? "unsupported_checksum" : hash.AsSpan().SequenceEqual(Pdb.GetBlobBytes(document.Hash)) ? "available" : "source_mismatch";
    }

    internal (uint Token, int Offset, int Line) Resolve(string source, int line, int? column = null)
    {
        if (line <= 0 || column is <= 0) throw new ArgumentException("Source coordinates must be positive.");
        var comparison = OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal;
        var points = Pdb.MethodDebugInformation.SelectMany(handle =>
        {
            var information = Pdb.GetMethodDebugInformation(handle);
            return information.GetSequencePoints().Where(point => !point.IsHidden).Select(point =>
                (Token: (uint)MetadataTokens.GetToken(handle.ToDefinitionHandle()), Point: point,
                 Source: Pdb.GetString(Pdb.GetDocument(point.Document.IsNil ? information.Document : point.Document).Name)));
        }).Where(item => string.Equals(System.IO.Path.GetFullPath(item.Source), System.IO.Path.GetFullPath(source), comparison)
            && (item.Point.StartLine > line || item.Point.StartLine == line && (column is null || item.Point.EndLine > line || item.Point.EndColumn > column)))
            .OrderBy(item => item.Point.StartLine).ThenBy(item => item.Point.StartColumn).ToArray();
        if (points.Length == 0) throw new ArgumentException("No executable source statement matches the requested breakpoint.");
        return (points[0].Token, points[0].Point.Offset, points[0].Point.StartLine);
    }

    internal IReadOnlyList<(string Name, int Slot, bool Cell)> Locals(uint methodToken, int offset)
    {
        var method = (MethodDefinitionHandle)MetadataTokens.Handle((int)methodToken);
        return Pdb.GetLocalScopes(method).Select(Pdb.GetLocalScope)
            .Where(scope => offset >= scope.StartOffset && offset < scope.EndOffset).OrderBy(scope => scope.Length)
            .SelectMany(scope => scope.GetLocalVariables()).Select(handle => (Handle: handle, Local: Pdb.GetLocalVariable(handle)))
            .Where(item => (item.Local.Attributes & LocalVariableAttributes.DebuggerHidden) == 0)
            .Select(item => (Name: Pdb.GetString(item.Local.Name), Slot: item.Local.Index, Cell: Storage(item.Handle)?.Cell == true)).DistinctBy(local => local.Name).ToArray();
    }

    internal int NextOffset(uint methodToken, int offset)
    {
        var method = (MethodDefinitionHandle)MetadataTokens.Handle((int)methodToken);
        return Pdb.GetMethodDebugInformation(method).GetSequencePoints().Where(point => !point.IsHidden && point.Offset > offset)
            .Select(point => point.Offset).DefaultIfEmpty(Image.GetMethodBody(Metadata.GetMethodDefinition(method).RelativeVirtualAddress).GetILReader().Length).Min();
    }

    public override void Dispose() { symbols.Dispose(); symbolStream.Dispose(); base.Dispose(); }
}
