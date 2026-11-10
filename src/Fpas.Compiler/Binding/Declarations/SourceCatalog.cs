using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bodies;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Binding.Semantics;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed class SourceUnit(SourceText source, CompilationUnitSyntax syntax)
{
    internal SourceText Source { get; } = source;
    internal CompilationUnitSyntax Syntax { get; } = syntax;
    internal Dictionary<string, VariantSymbol> Variants { get; } = new(StringComparer.Ordinal);
    internal IEnumerable<RecordSymbol> RecordTypes => Records.Values.Concat(Variants.Values.SelectMany(variant => variant.Cases).Select(member => member.Representation));
    internal Dictionary<string, RecordSymbol> Records { get; } = new(StringComparer.Ordinal);
    internal Dictionary<string, InterfaceSymbol> Interfaces { get; } = new(StringComparer.Ordinal);
    internal Dictionary<string, List<RoutineSymbol>> Routines { get; } = new(StringComparer.Ordinal);
    internal Dictionary<string, AliasSymbol> Aliases { get; } = new(StringComparer.Ordinal);
    internal Dictionary<string, ScalarSymbol> Scalars { get; } = new(StringComparer.Ordinal);
    internal Dictionary<string, GlobalSymbol> Globals { get; } = new(StringComparer.Ordinal);
    internal TypeSymbol ContainerType => new(TypeKind.Container, Container: this);
    internal RoutineSymbol? Initializer { get; set; }
    internal bool HasType(string name) => Records.ContainsKey(name) || Interfaces.ContainsKey(name) || Variants.ContainsKey(name) || Scalars.ContainsKey(name) || Aliases.ContainsKey(name);
    internal RoutineSymbol? EntryPoint { get; set; }
}

internal sealed partial class SourceCatalog
{
    internal SemanticRecorder? Analysis { get; private init; }
    internal int ClosureCount { get; set; }
    private readonly Dictionary<string, SourceUnit> names = new(StringComparer.Ordinal);
    internal IReadOnlyList<SourceUnit> Units { get; private init; } = [];

    internal static SourceCatalog Declare(IEnumerable<SourceUnit> sources, ReferenceCatalog references, SemanticRecorder? analysis = null)
    {
        var catalog = new SourceCatalog { Units = sources.OrderBy(unit => unit.Syntax.Name.Text, StringComparer.Ordinal).ToArray(), Analysis = analysis };
        var index = 0;
        foreach (var unit in catalog.Units)
        {
            unit.Source.CheckCancellation();
            var name = unit.Syntax.Name;
            if (!catalog.names.TryAdd(name.Text, unit))
                throw unit.Source.Error("FP2005", $"Source container '{name.Text}' is declared more than once.", name.Position, name.Text.Length,
                    related: [new("The earlier source container is here.", catalog.names[name.Text].Source.Location(catalog.names[name.Text].Syntax.Name.Position, name.Text.Length))]);
            if (references.HasType(name.Text))
                throw unit.Source.Error("FP4002", $"Source container '{name.Text}' conflicts with a referenced CLR type.", name.Position);
        }
        catalog.DeclareScalarNames();
        catalog.DeclareInterfaceNames();
        catalog.DeclareVariants(references);
        catalog.DeclareRecords(references);
        CompleteVariants(catalog.Units);
        catalog.DeclareGlobals(references);
        foreach (var unit in catalog.Units)
        {
            var aliases = unit.Syntax.Imports.Select(import => import.Alias.Text).ToHashSet(StringComparer.Ordinal);
            if (unit.Globals.Count != 0) unit.Initializer = new(".cctor", TypeSymbol.Void, [], false, false, index++);
            foreach (var routine in unit.Syntax.Routines)
            {
                if (unit.HasType(routine.Name.Text) || unit.Globals.ContainsKey(routine.Name.Text) || aliases.Contains(routine.Name.Text))
                    throw unit.Source.Error("FP3004", $"Routine '{routine.Name.Text}' conflicts with a type or import.", routine.Name.Position);
                AddOverload(unit.Routines, catalog.DeclareRoutine(unit, routine, references, index++), unit.Source, routine.Name.Position);
            }
            if (unit.Syntax.IsProgram)
                unit.EntryPoint = new(unit.Syntax.IsAsync ? "<AsyncProgram>" : "<EntryPoint>", unit.Syntax.IsAsync ? references.Symbol(references.Required("System.Threading.Tasks.Task")) : TypeSymbol.Void,
                    [new("args", TypeSymbol.Array(TypeSymbol.String), 0)], false, true, index++, IsAsync: unit.Syntax.IsAsync);
        }
        catalog.ValidateInterfaceContracts(references);
        catalog.BindRecordInterfaces(references);
        catalog.ValidateRecordVisibility();
        catalog.ValidateAliases(references);
        foreach (var unit in catalog.Units) new Binder(unit, catalog, references).BindDefaults();
        return catalog;
    }

    internal ImportTarget Resolve(string name, int position, SourceText source, ReferenceCatalog references)
    {
        source.CheckCancellation();
        var isNamespace = references.HasNamespace(name) || names.Keys.Any(key => key.StartsWith(name + ".", StringComparison.Ordinal));
        if (isNamespace && (names.ContainsKey(name) || references.HasType(name)))
            throw source.Error("FP4002", $"Import '{name}' is ambiguous between a namespace and a type.", position);
        if (!names.TryGetValue(name, out var unit))
        {
            foreach (var owner in Units)
            {
                if (name.StartsWith(owner.Syntax.Name.Text + ".", StringComparison.Ordinal) && owner.Interfaces.TryGetValue(name[(owner.Syntax.Name.Text.Length + 1)..], out var contract))
                {
                    if (!contract.IsPublic) throw source.Error("FP3029", $"Interface '{contract.Name}' is private to its unit.", position);
                    return new InterfaceImport(contract);
                }
                if (name.StartsWith(owner.Syntax.Name.Text + ".", StringComparison.Ordinal) && ResolveScalarPath(owner, name[(owner.Syntax.Name.Text.Length + 1)..], null, references, source, position) is { } scalar)
                    return scalar;
                if (name.StartsWith(owner.Syntax.Name.Text + ".", StringComparison.Ordinal) && owner.Variants.TryGetValue(name[(owner.Syntax.Name.Text.Length + 1)..], out var variant))
                {
                    if (!variant.IsPublic) throw source.Error("FP3023", $"Variant '{variant.Name}' is private to its unit.", position);
                    return new VariantImport(variant);
                }
                if (name.StartsWith(owner.Syntax.Name.Text + ".", StringComparison.Ordinal) && owner.Records.TryGetValue(name[(owner.Syntax.Name.Text.Length + 1)..], out var record))
                {
                    if (!record.IsPublic) throw source.Error("FP3021", $"Record '{record.Name}' is private to its unit.", position);
                    return new RecordImport(record);
                }
            }
            return isNamespace ? new NamespaceImport(name) : new ClrImport(references.FindType(name, position, source));
        }
        if (unit.Syntax.IsProgram)
            throw source.Error("FP3012", $"Program '{name}' cannot be imported; import a unit instead.", position);
        return new SourceImport(unit);
    }

    internal ImportTarget ResolvePath(IReadOnlyList<string> parts, SourceUnit unit, ReferenceCatalog references, int position, int arity = 0, bool allowOpenGeneric = false)
    {
        if (parts.Count == 1 && ResolveScalarPath(unit, parts[0], unit, references, unit.Source, position) is { } ownScalar) return ownScalar;
        if (parts.Count == 2 && unit.Variants.TryGetValue(parts[0], out var containingVariant))
            return SourceCaseType(containingVariant, parts[1], position, unit.Source);
        if (parts.Count == 1 && unit.Variants.TryGetValue(parts[0], out var ownVariant)) return new VariantImport(ownVariant);
        if (parts.Count == 1 && unit.Records.TryGetValue(parts[0], out var ownRecord)) return new RecordImport(ownRecord);
        if (parts.Count == 1 && unit.Interfaces.TryGetValue(parts[0], out var ownInterface)) return new InterfaceImport(ownInterface);
        var import = unit.Syntax.Imports.FirstOrDefault(candidate => candidate.Alias.Text == parts[0]);
        if (import is null)
        {
            var primitive = TypeFacts.Primitive(parts[0]);
            if (parts.Count == 1 && primitive != TypeSymbol.Unsupported) return new ClrImport(references.Definition(primitive));
            throw unit.Source.Error("FP3001", $"Unknown imported type or namespace '{parts[0]}'.", position);
        }
        var target = Resolve(import.Target.Text, position, unit.Source, references);
        if (parts.Count == 1) return target;
        if (target is SourceImport scalarUnit && parts.Count == 2 && ResolveScalarPath(scalarUnit.Unit, parts[1], unit, references, unit.Source, position) is { } scalar) return scalar;
        if (target is SourceImport interfaceUnit && parts.Count == 2 && interfaceUnit.Unit.Interfaces.TryGetValue(parts[1], out var contract))
        {
            if (!contract.IsPublic && interfaceUnit.Unit != unit) throw unit.Source.Error("FP3029", "The interface is private to its unit.", position);
            return new InterfaceImport(contract);
        }
        if (target is SourceImport caseOwner && parts.Count == 3 && caseOwner.Unit.Variants.TryGetValue(parts[1], out var caseVariant))
        {
            if (!caseVariant.IsPublic && caseOwner.Unit != unit) throw unit.Source.Error("FP3023", "The variant is private to its unit.", position);
            return SourceCaseType(caseVariant, parts[2], position, unit.Source);
        }
        if (target is SourceImport sourceVariant && parts.Count == 2 && sourceVariant.Unit.Variants.TryGetValue(parts[1], out var variant))
        {
            if (!variant.IsPublic && sourceVariant.Unit != unit) throw unit.Source.Error("FP3023", $"Variant '{variant.Name}' is private to its unit.", position);
            return new VariantImport(variant);
        }
        if (target is SourceImport sourceUnit && parts.Count == 2 && sourceUnit.Unit.Records.TryGetValue(parts[1], out var record))
        {
            if (!record.IsPublic && sourceUnit.Unit != unit) throw unit.Source.Error("FP3021", $"Record '{record.Name}' is private to its unit.", position);
            return new RecordImport(record);
        }
        if (target is VariantImport variantType && parts.Count == 2) return SourceCaseType(variantType.Variant, parts[1], position, unit.Source);
        if (target is ClrImport clr)
        {
            if (parts.Count == 2 && arity == 0 && references.Alias(references.Symbol(clr.Type), parts[1]) is { } alias)
            {
                Analysis?.MetadataAliasUse(unit.Source, position, references.Symbol(clr.Type), alias);
                return new ClrTypeImport(alias);
            }
            var name = clr.Type.FullName + "+" + string.Join('+', parts.Skip(1)) + (arity == 0 ? "" : "`" + arity);
            return new ClrImport(allowOpenGeneric && arity == 0 && !references.HasType(name) ? references.FindOpenType(name, position, unit.Source) : references.FindType(name, position, unit.Source));
        }
        if (target is NamespaceImport ns)
        {
            var name = ns.Namespace + "." + string.Join('.', parts.Skip(1));
            if (arity != 0 && references.HasType(name + "`" + arity)) name += "`" + arity;
            if (allowOpenGeneric && arity == 0 && !references.HasType(name) && !references.HasNamespace(name) &&
                !names.ContainsKey(name) && !names.Keys.Any(key => key.StartsWith(name + ".", StringComparison.Ordinal)))
                return new ClrImport(references.FindOpenType(name, position, unit.Source));
            return Resolve(name, position, unit.Source, references);
        }
        throw unit.Source.Error("FP3005", "Unsupported nested type path.", position);
    }

    private static ClrTypeImport SourceCaseType(VariantSymbol variant, string name, int position, SourceText source)
        => new(variant.Cases.FirstOrDefault(member => member.Representation.Name == name)?.Representation.Type
            ?? throw source.Error("FP3005", $"Unknown generated case type '{variant.Name}.{name}'.", position));

    internal TypeSymbol ResolveType(TypeSyntax syntax, SourceUnit unit, ReferenceCatalog references, IReadOnlyDictionary<string, GenericParameterSymbol>? generic = null, bool validate = true)
    {
        var type = ResolveTypeCore(syntax, unit, references, generic, validate);
        Analysis?.TypeSyntax(unit, syntax, type);
        return type;
    }

    private TypeSymbol ResolveTypeCore(TypeSyntax syntax, SourceUnit unit, ReferenceCatalog references, IReadOnlyDictionary<string, GenericParameterSymbol>? generic, bool validate)
    {
        unit.Source.CheckCancellation();
        TypeSymbol type;
        if (syntax.CallableParameters is not null)
            return references.Callable(syntax.CallableParameters.Select(parameter => ResolveType(parameter, unit, references, generic, validate)).ToArray(),
                syntax.CallableResult is null ? TypeSymbol.Void : ResolveType(syntax.CallableResult, unit, references, generic, validate), unit.Source, syntax.Position);
        if (syntax.Element is not null)
        {
            if (syntax.Rank is < 1 or > 32) throw unit.Source.Error("FP3005", "CLR arrays support ranks from 1 to 32.", syntax.Position);
            type = TypeSymbol.Array(ResolveType(syntax.Element, unit, references, generic, validate), syntax.Rank);
        }
        else
        {
            var primitive = syntax.Segments.Count == 1 ? TypeFacts.Primitive(syntax.Text) : TypeSymbol.Unsupported;
            var arguments = syntax.Segments[^1].Arguments;
            if (syntax.Segments.Count == 1 && generic?.TryGetValue(syntax.Text, out var parameter) == true)
            {
                if (arguments is not null) throw unit.Source.Error("FP3026", "A type parameter cannot have type arguments.", syntax.Position);
                if (validate && syntax.Nullable && !parameter.Type.IsValue && !parameter.Type.IsReference) throw unit.Source.Error("FP3026", "T? requires a reference or struct constraint.", syntax.Position);
                return parameter.Type with { IsNullable = syntax.Nullable };
            }
            if (primitive != TypeSymbol.Unsupported && arguments is null) type = primitive;
            else
            {
                if (syntax.Segments.Count == 1 && !unit.Syntax.Imports.Any(import => import.Alias.Text == syntax.Text) && !unit.HasType(syntax.Text))
                    throw unit.Source.Error("FP3005", $"Unknown or unsupported value type '{syntax.Text}'.", syntax.Position);
                var target = ResolveTypePath(syntax.Segments, unit, references, generic, validate);
                type = target switch
                {
                    RecordImport own => references.ConstructSource(own.Record.Type, [], unit.Source, syntax.Position, validate),
                    VariantImport own => references.ConstructSource(own.Variant.Type, [], unit.Source, syntax.Position, validate),
                    InterfaceImport own => references.ConstructSource(own.Interface.Type, [], unit.Source, syntax.Position, validate),
                    ClrTypeImport own when own.Type.Record is not null && own.Type.TypeArguments.Count == 0 => references.ConstructSource(own.Type, [], unit.Source, syntax.Position, validate),
                    ClrTypeImport own => own.Type,
                    ClrImport imported => references.Symbol(imported.Type),
                    _ => throw unit.Source.Error("FP3005", $"Unsupported value type '{syntax.Text}'.", syntax.Position)
                };
            }
        }
        if (!ReferenceCatalog.Usable(type) || type == TypeSymbol.Void || type.Definition?.FullName == "System.Void" ||
            type.Definition is not null && type.Definition.Definition.GetGenericParameters().Count != type.TypeArguments.Count)
            throw unit.Source.Error("FP3005", $"Unsupported value type '{syntax.Text}'.", syntax.Position);
        return type with { IsNullable = syntax.Nullable || type.IsNullable };
    }
}
