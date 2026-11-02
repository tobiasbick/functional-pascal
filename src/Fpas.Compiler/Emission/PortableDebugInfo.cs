using System.Collections.Immutable;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Security.Cryptography;
using System.Text;
using Fpas.Compiler.Binding;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Emission;

internal sealed class PortableDebugInfo
{
    private readonly MetadataBuilder metadata = new();
    private readonly Dictionary<string, DocumentHandle> documents = new(StringComparer.Ordinal);
    private readonly Dictionary<MethodDefinitionHandle, (DocumentHandle Document, BlobHandle Points)> methods = [];
    private readonly List<(MethodDefinitionHandle Method, IReadOnlyList<LocalSymbol> Locals, int Start, int Length)> scopes = [];
    private readonly List<(MethodDefinitionHandle Move, MethodDefinitionHandle Kickoff, IReadOnlyList<(int Yield, int Resume)> Points, IReadOnlyList<LocalSymbol> Locals, int Length)> asyncMethods = [];

    internal void AddAsync(MethodDefinitionHandle move, MethodDefinitionHandle kickoff, IReadOnlyList<(int Yield, int Resume)> points, IReadOnlyList<LocalSymbol> locals, int length)
        => asyncMethods.Add((move, kickoff, points, locals, length));

    internal PortableDebugInfo(IEnumerable<SourceText> sources)
    {
        foreach (var source in sources.OrderBy(source => source.Path, StringComparer.Ordinal))
            documents.Add(source.Path, metadata.AddDocument(metadata.GetOrAddDocumentName(source.Path),
                metadata.GetOrAddGuid(new("8829d00f-11b8-4213-878b-770e8597ac16")),
                metadata.GetOrAddBlob(SHA256.HashData(Encoding.UTF8.GetBytes(source.Text))), default));
    }

    internal void AddMethod(MethodDefinitionHandle method, IReadOnlyList<(int Offset, SourceLocation? Location)> points,
        IReadOnlyList<LocalSymbol> locals, StandaloneSignatureHandle signature, int length,
        IReadOnlyList<(int Start, int Length, IReadOnlyList<LocalSymbol> Locals)>? lexicalScopes = null)
    {
        var visible = points.FirstOrDefault(point => point.Location is not null).Location;
        if (visible is null) return;
        var document = documents[visible.Path];
        var blob = new BlobBuilder();
        blob.WriteCompressedInteger(MetadataTokens.GetRowNumber(signature));
        var previousOffset = 0; var previousLine = 0; var previousColumn = 0;
        var first = true;
        foreach (var (offset, location) in points)
        {
            if (offset >= length) continue;
            blob.WriteCompressedInteger(first ? offset : offset - previousOffset);
            previousOffset = offset; first = false;
            if (location is null)
            {
                blob.WriteCompressedInteger(0); blob.WriteCompressedInteger(0);
                continue;
            }
            blob.WriteCompressedInteger(0); blob.WriteCompressedInteger(Math.Max(1, location.Length));
            if (previousLine == 0) { blob.WriteCompressedInteger(location.Line); blob.WriteCompressedInteger(location.Column); }
            else { blob.WriteCompressedSignedInteger(location.Line - previousLine); blob.WriteCompressedSignedInteger(location.Column - previousColumn); }
            previousLine = location.Line; previousColumn = location.Column;
        }
        methods.Add(method, (document, metadata.GetOrAddBlob(blob)));
        var assigned = new HashSet<LocalSymbol>();
        foreach (var scope in (lexicalScopes ?? []).OrderByDescending(scope => scope.Length))
        {
            var unique = scope.Locals.Where(assigned.Add).ToArray();
            if (unique.Length != 0) scopes.Add((method, unique, scope.Start, scope.Length));
        }
        var remaining = locals.Where(local => !assigned.Contains(local) && !local.Name.StartsWith('<')).ToArray();
        if (remaining.Length != 0) scopes.Add((method, remaining, 0, length));
    }

    internal (byte[] Image, BlobContentId Id) Build(MetadataBuilder assembly, MethodDefinitionHandle entry)
    {
        for (var row = 1; row <= assembly.GetRowCount(TableIndex.MethodDef); row++)
        {
            var method = MetadataTokens.MethodDefinitionHandle(row);
            if (methods.TryGetValue(method, out var data)) metadata.AddMethodDebugInformation(data.Document, data.Points);
            else metadata.AddMethodDebugInformation(default, default);
        }
        foreach (var scope in scopes.OrderBy(scope => MetadataTokens.GetRowNumber(scope.Method)).ThenBy(scope => scope.Start).ThenByDescending(scope => scope.Length))
        {
            var first = MetadataTokens.LocalVariableHandle(metadata.GetRowCount(TableIndex.LocalVariable) + 1);
            foreach (var local in scope.Locals.Where(local => !local.Name.StartsWith('<')))
                metadata.AddLocalVariable(LocalVariableAttributes.None, local.Slot, metadata.GetOrAddString(local.Name));
            metadata.AddLocalScope(scope.Method, default, first, MetadataTokens.LocalConstantHandle(1), scope.Start, scope.Length);
        }
        foreach (var method in asyncMethods.OrderBy(method => MetadataTokens.GetRowNumber(method.Move)))
        {
            metadata.AddStateMachineMethod(method.Move, method.Kickoff);
            var stepping = new BlobBuilder(); stepping.WriteUInt32(0);
            foreach (var (yield, resume) in method.Points)
            {
                stepping.WriteUInt32((uint)yield); stepping.WriteUInt32((uint)resume);
                stepping.WriteCompressedInteger(MetadataTokens.GetRowNumber(method.Move));
            }
            metadata.AddCustomDebugInformation(method.Move, metadata.GetOrAddGuid(new("54FD2AC5-E925-401A-9C2A-F94F171072F8")), metadata.GetOrAddBlob(stepping));
            var hoisted = new BlobBuilder();
            foreach (var local in method.Locals)
            {
                hoisted.WriteUInt32(0); hoisted.WriteUInt32(local.Type.Kind == TypeKind.ByRef ? 0 : (uint)method.Length);
            }
            metadata.AddCustomDebugInformation(method.Move, metadata.GetOrAddGuid(new("6DA9A61E-F8C7-4874-BE62-68BC5630DF71")), metadata.GetOrAddBlob(hoisted));
        }
        var builder = new PortablePdbBuilder(metadata, assembly.GetRowCounts(), entry, content =>
        {
            using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            foreach (var blob in content) hash.AppendData(blob.GetBytes());
            return BlobContentId.FromHash(hash.GetHashAndReset());
        });
        var image = new BlobBuilder();
        var id = builder.Serialize(image);
        return (image.ToArray(), id);
    }
}
