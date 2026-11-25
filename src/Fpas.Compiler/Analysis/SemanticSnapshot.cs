// [Documentation](../../../docs/architecture/compiler.md#semantic-snapshots)
namespace Fpas.Compiler.Analysis;

/// <summary>The CLR identity of an imported or emitted type or member.</summary>
/// <param name="Assembly">The simple assembly name.</param>
/// <param name="Version">The assembly version.</param>
/// <param name="Culture">The assembly culture, empty for neutral.</param>
/// <param name="PublicKeyToken">The hexadecimal public key token, empty when unsigned.</param>
/// <param name="Type">The CLR full name of the type or declaring type.</param>
/// <param name="Kind">The member kind, such as type, method, property, or field.</param>
/// <param name="Name">The CLR member name.</param>
/// <param name="Arity">The generic arity.</param>
/// <param name="Parameters">The CLR parameter type names.</param>
/// <param name="Result">The CLR result type name, when the member has one.</param>
/// <param name="AssemblyHash">The SHA-256 content hash of the defining reference, when captured.</param>
public sealed record MetadataIdentity(string Assembly, string Version, string Culture, string PublicKeyToken,
    string Type, string Kind, string Name, int Arity, IReadOnlyList<string> Parameters, string? Result, string? AssemblyHash = null)
{
    // Exact MethodDef selected by the binder within AssemblyHash.
    internal int DefinitionToken { get; init; }

    /// <summary>Builds a stable string key for this identity.</summary>
    /// <param name="includeAssemblyHash">Whether the key distinguishes reference contents.</param>
    /// <returns>The identity key.</returns>
    public string GetKey(bool includeAssemblyHash = true) => Assembly + "/" + Version + "/" + Culture + "/" + PublicKeyToken + ":" + Type + ":" + Kind + ":" + Name +
        "`" + Arity.ToString(System.Globalization.CultureInfo.InvariantCulture) + "(" + string.Join(',', Parameters) + "):" + Result + (includeAssemblyHash ? ":" + AssemblyHash : "");
}
/// <summary>Links a source symbol to the CLR identity it is emitted as.</summary>
/// <param name="Identity">The emitted CLR identity.</param>
/// <param name="Symbol">The snapshot symbol id.</param>
public sealed record SymbolExport(MetadataIdentity Identity, string Symbol);

/// <summary>A resolved import in the checked source.</summary>
/// <param name="Alias">The alias declaration's symbol id.</param>
/// <param name="Target">The imported unit, type, or namespace symbol id.</param>
/// <param name="IsClr">Whether the target is an imported CLR type or namespace.</param>
public sealed record SemanticImport(string Alias, string Target, bool IsClr);

/// <summary>One parameter of a callable signature.</summary>
/// <param name="Name">The parameter name.</param>
/// <param name="Type">The snapshot type id.</param>
/// <param name="Mode">The passing mode: empty for a value parameter, or in, out, ref, or params.</param>
public sealed record SemanticParameter(string Name, string Type, string Mode);

/// <summary>The checked signature of a routine, method, or delegate.</summary>
/// <param name="Parameters">The parameters in declaration order.</param>
/// <param name="Result">The snapshot type id of the result.</param>
/// <param name="TypeParameters">The snapshot type ids of the method type parameters.</param>
public sealed record CallableSignature(IReadOnlyList<SemanticParameter> Parameters, string Result, IReadOnlyList<string> TypeParameters);

/// <summary>A checked type referenced by the snapshot.</summary>
/// <param name="Id">The snapshot type id.</param>
/// <param name="Kind">The type category, such as primitive, array, record, or type parameter.</param>
/// <param name="Display">The FPAS display spelling.</param>
/// <param name="Nullable">Whether the type is a nullable form.</param>
/// <param name="Rank">The array rank, or zero for a non-array type.</param>
/// <param name="Element">The type id of the array element, when an array.</param>
/// <param name="Arguments">The type ids of generic arguments.</param>
/// <param name="Parameter">The symbol id of the generic parameter this type denotes, when one.</param>
/// <param name="Metadata">The CLR identity of the type definition, when available.</param>
/// <param name="Callable">The delegate signature, when the type is callable.</param>
/// <param name="GenericIndex">The position of a generic parameter in its owner.</param>
/// <param name="GenericOwner">The owner of a generic parameter.</param>
/// <param name="IsValue">Whether the type is a CLR value type.</param>
/// <param name="IsReference">Whether the type is a CLR reference type.</param>
/// <param name="MayBeNull">Whether a value of the type may be null.</param>
public sealed record SemanticType(string Id, string Kind, string Display, bool Nullable, int Rank, string? Element,
    IReadOnlyList<string> Arguments, string? Parameter, MetadataIdentity? Metadata, CallableSignature? Callable,
    int? GenericIndex = null, string? GenericOwner = null, bool IsValue = false, bool IsReference = false, bool MayBeNull = false);

/// <summary>A declared or imported symbol.</summary>
/// <param name="Id">The snapshot symbol id.</param>
/// <param name="Kind">The symbol category, such as routine, local, parameter, type, alias, or namespace.</param>
/// <param name="Name">The declared name.</param>
/// <param name="Signature">The rendered declaration signature.</param>
/// <param name="Declaration">The source name span, for a source symbol.</param>
/// <param name="Type">The type id of the symbol's value, when it has one.</param>
/// <param name="Metadata">The CLR identity, for an imported symbol.</param>
/// <param name="Callable">The checked signature, for a callable symbol.</param>
/// <param name="Context">The declaration surroundings, for a source declaration.</param>
public sealed record SemanticSymbol(string Id, string Kind, string Name, string Signature, SourceSpan? Declaration,
    string? Type, MetadataIdentity? Metadata, CallableSignature? Callable, DeclarationContext? Context = null);

/// <summary>Where a source declaration lives and what its surface mentions.</summary>
/// <param name="Owner">The symbol id of the enclosing declaration, when nested.</param>
/// <param name="QualifiedName">The qualified source name.</param>
/// <param name="Accessibility">The declared accessibility, when it has one.</param>
/// <param name="Span">The span of the whole declaration.</param>
/// <param name="Body">The span of the body, when it has one.</param>
/// <param name="Documentation">The span of the attached documentation comment.</param>
/// <param name="SurfaceTypes">The type ids mentioned by the declaration's signature.</param>
/// <param name="SurfaceSymbols">The symbol ids mentioned by the declaration's signature.</param>
/// <param name="IsInstance">Whether the declaration is an instance routine.</param>
/// <param name="HasProtocolBases">Whether the protocol extends any interfaces.</param>
public sealed record DeclarationContext(string? Owner, string QualifiedName, string? Accessibility, SourceSpan Span,
    SourceSpan? Body, SourceSpan? Documentation, IReadOnlyList<string> SurfaceTypes, IReadOnlyList<string> SurfaceSymbols,
    bool IsInstance = false, bool HasProtocolBases = false);

/// <summary>One source mention of a symbol.</summary>
/// <param name="Symbol">The snapshot symbol id.</param>
/// <param name="Span">The mentioned name.</param>
/// <param name="Role">How the mention uses the symbol, such as declaration, read, write, or read_write.</param>
public sealed record SymbolOccurrence(string Symbol, SourceSpan Span, string Role)
{
    // The actual FieldDef or accessor MethodDef for this access, including ordinary storage outside the call graph.
    internal int AccessToken { get; init; }
}

/// <summary>A property used as a receiver for a call, index, or write through its value.</summary>
/// <param name="Property">The property symbol id.</param>
/// <param name="Span">Source range of the property access.</param>
public sealed record PropertyReach(string Property, SourceSpan Span);

/// <summary>Records that a member implements a protocol member.</summary>
/// <param name="Implementation">The symbol id of the implementing member.</param>
/// <param name="Contract">The symbol id of the implemented member.</param>
public sealed record SymbolRelation(string Implementation, string Contract);

/// <summary>The checked type of a source expression.</summary>
/// <param name="Span">The expression range.</param>
/// <param name="Type">The type id after flow analysis.</param>
/// <param name="DeclaredType">The declared type id, when it differs from <paramref name="Type"/>.</param>
public sealed record ExpressionType(SourceSpan Span, string Type, string? DeclaredType);

/// <summary>Checked semantic facts captured by one analysis.</summary>
public sealed class SemanticSnapshot
{
    /// <summary>The snapshot identity.</summary>
    public string Id { get; }
    /// <summary>The analyzed assembly name.</summary>
    public string AssemblyName { get; }
    /// <summary>The captured sources.</summary>
    public IReadOnlyList<SourceDocument> Documents { get; }
    /// <summary>All declared and referenced symbols.</summary>
    public IReadOnlyList<SemanticSymbol> Symbols { get; }
    /// <summary>All referenced types.</summary>
    public IReadOnlyList<SemanticType> Types { get; }
    /// <summary>Symbol mentions in source order.</summary>
    public IReadOnlyList<SymbolOccurrence> Occurrences { get; }
    /// <summary>Resolved import aliases and their targets.</summary>
    public IReadOnlyList<SemanticImport> Imports { get; }
    /// <summary>Direct access through property values used as receivers.</summary>
    public IReadOnlyList<PropertyReach> PropertyReaches { get; }
    /// <summary>Checked expression types.</summary>
    public IReadOnlyList<ExpressionType> Expressions { get; }
    /// <summary>The CLR identities of emitted source symbols.</summary>
    public IReadOnlyList<SymbolExport> Exports { get; }
    /// <summary>Routine, lambda, and initializer bodies.</summary>
    public IReadOnlyList<SourceBody> Bodies { get; }
    /// <summary>Calls, constructions, and other operations inside bodies.</summary>
    public IReadOnlyList<SourceOperation> Operations { get; }
    /// <summary>Operations whose targets the snapshot does not model.</summary>
    public IReadOnlyList<ExcludedOperation> ExcludedOperations { get; }
    /// <summary>Protocol implementation relations.</summary>
    public IReadOnlyList<SymbolRelation> Relations { get; }
    /// <summary>Which source regions have checked facts.</summary>
    public SemanticCoverage Coverage { get; }
    // Symbol lookup is a hot path for editor queries that resolve every occurrence.
    private readonly Dictionary<string, SemanticSymbol> symbolsById = new(StringComparer.Ordinal);

    internal SemanticSnapshot(string id, string assemblyName, IEnumerable<SourceDocument> documents, IEnumerable<SemanticSymbol> symbols,
        IEnumerable<SemanticType> types, IEnumerable<SymbolOccurrence> occurrences, IEnumerable<SemanticImport> imports, IEnumerable<PropertyReach> propertyReaches,
        IEnumerable<ExpressionType> expressions, IEnumerable<SymbolExport> exports,
        IEnumerable<SourceBody> bodies, IEnumerable<SourceOperation> operations, IEnumerable<ExcludedOperation> excludedOperations,
        IEnumerable<SymbolRelation>? relations = null, IEnumerable<SemanticRegion>? invalid = null)
    {
        Id = id; AssemblyName = assemblyName; Documents = Array.AsReadOnly(documents.ToArray()); Symbols = Array.AsReadOnly(symbols.ToArray());
        Types = Array.AsReadOnly(types.ToArray()); Occurrences = Array.AsReadOnly(occurrences.ToArray()); Expressions = Array.AsReadOnly(expressions.ToArray());
        Imports = Array.AsReadOnly(imports.ToArray());
        PropertyReaches = Array.AsReadOnly(propertyReaches.ToArray());
        Exports = Array.AsReadOnly(exports.ToArray());
        Bodies = Array.AsReadOnly(bodies.ToArray()); Operations = Array.AsReadOnly(operations.ToArray());
        ExcludedOperations = Array.AsReadOnly(excludedOperations.ToArray());
        Relations = Array.AsReadOnly((relations ?? []).ToArray());
        Coverage = new(Documents, invalid ?? []);
        foreach (var symbol in Symbols) symbolsById.TryAdd(symbol.Id, symbol);
    }

    /// <summary>Looks up a symbol by id.</summary>
    /// <param name="id">The snapshot symbol id.</param>
    /// <returns>The symbol.</returns>
    /// <exception cref="ArgumentException"><paramref name="id"/> does not belong to this snapshot.</exception>
    public SemanticSymbol Symbol(string id) => symbolsById.GetValueOrDefault(id)
        ?? throw new ArgumentException("Symbol does not belong to this snapshot.", nameof(id));

    /// <summary>Finds the innermost symbol mention at a cursor, preferring a declaration.</summary>
    /// <param name="path">The source path.</param>
    /// <param name="offset">The zero-based offset.</param>
    /// <returns>The mention, or <see langword="null"/> when none covers the offset.</returns>
    public SymbolOccurrence? OccurrenceAt(string path, int offset)
        => Occurrences.Where(occurrence => SamePath(occurrence.Span.Path, path) && occurrence.Span.Contains(offset))
            .OrderBy(occurrence => occurrence.Span.Length).ThenBy(occurrence => occurrence.Role == "declaration" ? 0 : 1).FirstOrDefault();

    /// <summary>Finds the innermost checked expression type at a cursor.</summary>
    /// <param name="path">The source path.</param>
    /// <param name="offset">The zero-based offset.</param>
    /// <returns>The expression type, or <see langword="null"/> for none or for a namespace or type alias name.</returns>
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
