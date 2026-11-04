using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;
using System.Security.Cryptography;
using System.Runtime.InteropServices;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder(string assemblyName, ReferenceCatalog references, string snapshotId, CancellationToken token)
{
    private sealed class SymbolEntry(object identity, string kind, string name)
    {
        internal object Identity { get; } = identity;
        internal string Id { get; set; } = "";
        internal string Kind { get; set; } = kind;
        internal string Name { get; set; } = name;
        internal SourceSpan? Declaration { get; set; }
        internal string? Type { get; set; }
        internal MetadataIdentity? Metadata { get; set; }
        internal CallableSignature? Callable { get; set; }
        internal string Signature { get; set; } = name;
    }

    private readonly Dictionary<object, SymbolEntry> symbols = new(ReferenceEqualityComparer.Instance);
    private readonly Dictionary<string, SymbolEntry> metadataSymbols = new(StringComparer.Ordinal);
    private readonly Dictionary<string, MetadataIdentity> metadataIdentities = new(StringComparer.Ordinal);
    private readonly Dictionary<string, SourceDocument> documents = new(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
    private readonly Dictionary<string, IReadOnlyDictionary<int, Token>> tokens = new(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
    private readonly Dictionary<(string Path, int Offset, string Role), (SymbolEntry Symbol, SourceSpan Span)> occurrences = [];
    private readonly Dictionary<(string Path, int Offset, int Length), ExpressionType> expressions = [];
    private readonly Dictionary<TypeSymbol, string> typeIds = [];
    private readonly Dictionary<string, SemanticType> types = new(StringComparer.Ordinal);
    private readonly Dictionary<GenericParameterSymbol, string> parameterKinds = new(ReferenceEqualityComparer.Instance);

    internal void Source(SourceText source, IReadOnlyList<Token> sourceTokens)
    {
        documents.Add(source.Path, new(source.Path, source.Text, token));
        tokens.Add(source.Path, sourceTokens.Where(value => value.Kind != TokenKind.End).ToDictionary(value => value.Position));
    }

    internal static string Fingerprint(AnalysisRequest request, ReferenceCatalog references, CancellationToken token)
    {
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        void Add(string text) { hash.AppendData(BitConverter.GetBytes(text.Length)); hash.AppendData(MemoryMarshal.AsBytes(text.AsSpan())); }
        Add(request.AssemblyName); Add(request.Target.ToString()); Add(request.ValidateDocumentation.ToString());
        var policy = request.DiagnosticOptions ?? new(); Add(policy.TreatWarningsAsErrors.ToString());
        foreach (var group in new[] { policy.WarningsAsErrors, policy.WarningsNotAsErrors, policy.NoWarn })
        { if (group is not null) for (var index = 0; index < group.Count; index++) Add(group[index]); Add("|"); }
        foreach (var source in request.Sources.OrderBy(source => source.Path, StringComparer.Ordinal)) { token.ThrowIfCancellationRequested(); Add(source.Path); Add(source.Text); }
        foreach (var (path, fingerprint) in references.InputFingerprints.OrderBy(item => item.Key, StringComparer.Ordinal))
        {
            token.ThrowIfCancellationRequested(); Add(path); Add(fingerprint);
        }
        Add(typeof(FpasCompiler).Assembly.ManifestModule.ModuleVersionId.ToString("D"));
        return Convert.ToHexString(hash.GetHashAndReset());
    }

    private SymbolEntry Entry(object identity, string kind, string name)
    {
        token.ThrowIfCancellationRequested();
        if (!symbols.TryGetValue(identity, out var entry))
        {
            entry = new(identity, kind, name) { Id = snapshotId + ":s" + symbols.Count.ToString(System.Globalization.CultureInfo.InvariantCulture) };
            symbols.Add(identity, entry);
        }
        return entry;
    }

    private SourceSpan Span(SourceText source, int position, int length) => documents[source.Path].Span(position, length);
    private Token? TokenAt(SourceText source, int position) => tokens[source.Path].GetValueOrDefault(position);
    private void Occurrence(SourceText source, Token name, SymbolEntry symbol, string role)
        => occurrences[(source.Path, name.Position, role)] = (symbol, Span(source, name.Position, name.Length));

    private void Declare(object identity, string kind, string name, SourceText source, Token nameToken, TypeSymbol? type = null, MetadataIdentity? metadata = null, CallableSignature? callable = null)
    {
        var original = symbols.GetValueOrDefault(identity);
        var resolved = metadata is null ? null : metadataSymbols.GetValueOrDefault(Key(metadata));
        var entry = original ?? resolved ?? Entry(identity, kind, name);
        if (resolved is not null && !ReferenceEquals(resolved, entry)) Merge(resolved, entry);
        symbols[identity] = entry;
        entry.Kind = kind; entry.Name = name;
        entry.Declaration ??= Span(source, nameToken.Position, nameToken.Length);
        entry.Type = type is null ? null : Type(type); entry.Metadata = metadata; entry.Callable = callable;
        entry.Signature = callable is null ? name + (type is null ? "" : ": " + TypeFacts.Display(type))
            : name + "(" + string.Join("; ", callable.Parameters.Select(parameter => parameter.Mode + " " + parameter.Name + ": " + types[parameter.Type].Display)) + "): " + types[callable.Result].Display;
        if (metadata is not null) Export(metadata, entry);
        Occurrence(source, nameToken, entry, "declaration");
        if (callable is not null)
        {
            var callableId = snapshotId + ":c" + entry.Id;
            types[callableId] = new(callableId, "callable", entry.Signature, false, 0, null, Array.Empty<string>(), null, null, callable);
            entry.Type = callableId;
        }
        if (entry.Type is not null) Typed(source, nameToken.Position, nameToken.Length, entry.Type);
    }

    private void Typed(SourceText source, int offset, int length, string type, string? declared = null)
    {
        if (length <= 0) return;
        var span = Span(source, offset, length);
        expressions[(source.Path, offset, length)] = new(span, type, declared == type ? null : declared);
    }

    internal void Local(LocalSymbol local, SourceText source, int position)
    {
        if (TokenAt(source, position) is { } name) Declare(local, "local", name.Text, source, name, local.Type);
    }

    internal SemanticSnapshot Finish()
    {
        token.ThrowIfCancellationRequested();
        var allSymbols = symbols.Values.Distinct().Select(entry => new SemanticSymbol(entry.Id, entry.Kind, entry.Name, entry.Signature, entry.Declaration,
            entry.Type, entry.Metadata, entry.Callable)).OrderBy(symbol => symbol.Id, StringComparer.Ordinal).ToArray();
        return new(snapshotId, assemblyName, documents.Values.OrderBy(document => document.Path, StringComparer.Ordinal), allSymbols,
            types.Values.OrderBy(type => type.Id, StringComparer.Ordinal),
            occurrences.Select(item => new SymbolOccurrence(item.Value.Symbol.Id, item.Value.Span, item.Key.Role)).Distinct()
                .OrderBy(item => item.Span.Path, StringComparer.Ordinal).ThenBy(item => item.Span.Offset).ThenBy(item => item.Role, StringComparer.Ordinal),
            expressions.Values.OrderBy(item => item.Span.Path, StringComparer.Ordinal).ThenBy(item => item.Span.Offset).ThenBy(item => item.Span.Length),
            metadataIdentities.OrderBy(item => item.Key, StringComparer.Ordinal).Select(item => new SymbolExport(item.Value, metadataSymbols[item.Key].Id)));
    }

    private void Export(MetadataIdentity identity, SymbolEntry entry)
    {
        var key = Key(identity); metadataSymbols[key] = entry; metadataIdentities[key] = identity;
    }

    private void Merge(SymbolEntry previous, SymbolEntry current)
    {
        foreach (var identity in symbols.Where(item => ReferenceEquals(item.Value, previous)).Select(item => item.Key).ToArray()) symbols[identity] = current;
        foreach (var key in metadataSymbols.Where(item => ReferenceEquals(item.Value, previous)).Select(item => item.Key).ToArray()) metadataSymbols[key] = current;
        foreach (var (key, occurrence) in occurrences.Where(item => ReferenceEquals(item.Value.Symbol, previous)).ToArray())
            occurrences[key] = (current, occurrence.Span);
    }
}
