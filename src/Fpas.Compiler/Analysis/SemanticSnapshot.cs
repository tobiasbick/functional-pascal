namespace Fpas.Compiler.Analysis;

public sealed record MetadataIdentity(string Assembly, string Version, string Culture, string PublicKeyToken,
    string Type, string Kind, string Name, int Arity, IReadOnlyList<string> Parameters, string? Result, string? AssemblyHash = null)
{
    public string GetKey(bool includeAssemblyHash = true) => Assembly + "/" + Version + "/" + Culture + "/" + PublicKeyToken + ":" + Type + ":" + Kind + ":" + Name +
        "`" + Arity.ToString(System.Globalization.CultureInfo.InvariantCulture) + "(" + string.Join(',', Parameters) + "):" + Result + (includeAssemblyHash ? ":" + AssemblyHash : "");
}
public sealed record SymbolExport(MetadataIdentity Identity, string Symbol);
public sealed record SemanticParameter(string Name, string Type, string Mode);
public sealed record CallableSignature(IReadOnlyList<SemanticParameter> Parameters, string Result, IReadOnlyList<string> TypeParameters);
public sealed record SemanticType(string Id, string Kind, string Display, bool Nullable, int Rank, string? Element,
    IReadOnlyList<string> Arguments, string? Parameter, MetadataIdentity? Metadata, CallableSignature? Callable,
    int? GenericIndex = null, string? GenericOwner = null, bool IsValue = false, bool IsReference = false, bool MayBeNull = false);
public sealed record SemanticSymbol(string Id, string Kind, string Name, string Signature, SourceSpan? Declaration,
    string? Type, MetadataIdentity? Metadata, CallableSignature? Callable);
public sealed record SymbolOccurrence(string Symbol, SourceSpan Span, string Role);
public sealed record ExpressionType(SourceSpan Span, string Type, string? DeclaredType);

public sealed class SemanticSnapshot
{
    public string Id { get; }
    public string AssemblyName { get; }
    public IReadOnlyList<SourceDocument> Documents { get; }
    public IReadOnlyList<SemanticSymbol> Symbols { get; }
    public IReadOnlyList<SemanticType> Types { get; }
    public IReadOnlyList<SymbolOccurrence> Occurrences { get; }
    public IReadOnlyList<ExpressionType> Expressions { get; }
    public IReadOnlyList<SymbolExport> Exports { get; }

    internal SemanticSnapshot(string id, string assemblyName, IEnumerable<SourceDocument> documents, IEnumerable<SemanticSymbol> symbols,
        IEnumerable<SemanticType> types, IEnumerable<SymbolOccurrence> occurrences, IEnumerable<ExpressionType> expressions, IEnumerable<SymbolExport> exports)
    {
        Id = id; AssemblyName = assemblyName; Documents = Array.AsReadOnly(documents.ToArray()); Symbols = Array.AsReadOnly(symbols.ToArray());
        Types = Array.AsReadOnly(types.ToArray()); Occurrences = Array.AsReadOnly(occurrences.ToArray()); Expressions = Array.AsReadOnly(expressions.ToArray());
        Exports = Array.AsReadOnly(exports.ToArray());
    }

    public SemanticSymbol Symbol(string id) => Symbols.FirstOrDefault(symbol => symbol.Id == id)
        ?? throw new ArgumentException("Symbol does not belong to this snapshot.", nameof(id));

    public SymbolOccurrence? OccurrenceAt(string path, int offset)
        => Occurrences.Where(occurrence => SamePath(occurrence.Span.Path, path) && occurrence.Span.Contains(offset))
            .OrderBy(occurrence => occurrence.Span.Length).ThenBy(occurrence => occurrence.Role == "declaration" ? 0 : 1).FirstOrDefault();

    public ExpressionType? TypeAt(string path, int offset)
    {
        var occurrence = OccurrenceAt(path, offset);
        var selected = occurrence is null ? null : Symbol(occurrence.Symbol);
        if (selected?.Kind == "namespace" || selected is { Kind: "alias", Type: null }) return null;
        return Expressions.Where(expression => SamePath(expression.Span.Path, path) && expression.Span.Contains(offset))
            .OrderBy(expression => expression.Span.Length).ThenByDescending(expression => expression.Span.Offset).FirstOrDefault();
    }

    private static bool SamePath(string left, string right) => string.Equals(left, right, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal);
}
