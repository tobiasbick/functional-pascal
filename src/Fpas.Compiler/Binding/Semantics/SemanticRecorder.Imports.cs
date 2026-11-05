using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private readonly Dictionary<string, object> namespaces = new(StringComparer.Ordinal);

    private SymbolEntry Namespace(string name)
    {
        if (!namespaces.TryGetValue(name, out var identity)) namespaces.Add(name, identity = new object());
        return Entry(identity, "namespace", name);
    }

    private SymbolEntry TypeSymbolEntry(TypeSymbol type)
    {
        if (type.Parameter is { } parameter) return Entry(parameter, "type_parameter", parameter.Name);
        if (type.Record is { } record) return Entry(record, "record", record.Name);
        if (type.Union is { } union) return Entry(union, "union", union.Name);
        if (type.Interface is { } contract) return Entry(contract, "interface", contract.Name);
        if (type.Scalar is { } scalar) return Entry(scalar, "domain", scalar.Name);
        if (type.Container is { } container) return Entry(container, "container", container.Syntax.Name.Text);
        var identity = TypeMetadata(type);
        return External(identity, "type", identity.Type);
    }

    private TypeSymbol? ImportedType(ImportTarget target) => target switch
    {
        ClrImport imported => references.Symbol(imported.Type), ClrTypeImport imported => imported.Type,
        RecordImport imported => imported.Record.Type, UnionImport imported => imported.Union.Type, InterfaceImport imported => imported.Interface.Type,
        _ => null
    };

    private SymbolEntry ImportTargetSymbol(ImportTarget target) => target switch
    {
        NamespaceImport ns => Namespace(ns.Namespace), SourceImport source => Entry(source.Unit, "container", source.Unit.Syntax.Name.Text),
        _ when ImportedType(target) is { } type => TypeSymbolEntry(type),
        _ => throw new InvalidOperationException("Unrecognized import target.")
    };

    internal void Import(SourceUnit unit, ImportSyntax syntax, ImportTarget target)
    {
        var type = ImportedType(target);
        Declare(syntax, "alias", syntax.Alias.Text, unit.Source, syntax.Alias, type);
        var symbol = ImportTargetSymbol(target);
        Occurrence(unit.Source, syntax.Target.Parts[^1], symbol, type is null ? "namespace" : "type");
        for (var index = 0; index < syntax.Target.Parts.Count - 1; index++)
            Occurrence(unit.Source, syntax.Target.Parts[index], Prefix(string.Join('.', syntax.Target.Parts.Take(index + 1).Select(part => part.Text)), target), "namespace");
    }

    private SymbolEntry Prefix(string name, ImportTarget target)
    {
        var type = ImportedType(target);
        var source = type?.Record?.Owner ?? type?.Union?.Owner ?? type?.Interface?.Owner ?? type?.Scalar?.Owner;
        if (source is not null && source.Syntax.Name.Text == name) return TypeSymbolEntry(source.ContainerType);
        for (var parent = type?.Definition?.DeclaringType; parent is not null; parent = parent.DeclaringType)
            if (System.Text.RegularExpressions.Regex.Replace(parent.FullName.Replace('+', '.'), @"`\d+", "") == name)
                return TypeSymbolEntry(references.Symbol(parent));
        return Namespace(name);
    }

    internal void TypePart(SourceUnit unit, Token name, ImportTarget target, bool first)
    {
        var key = (unit.Source.Path, name.Position, "type");
        // Transparent aliases retain their source declaration rather than the erased type.
        if (occurrences.TryGetValue(key, out var previous) && previous.Symbol.Kind == "alias") return;
        var import = first ? unit.Syntax.Imports.FirstOrDefault(import => import.Alias.Text == name.Text) : null;
        var symbol = import is not null ? Entry(import, "alias", import.Alias.Text) : ImportTargetSymbol(target);
        var type = ImportedType(target);
        Occurrence(unit.Source, name, symbol, type is null ? "namespace" : "type");
        if (type is not null) Typed(unit.Source, name.Position, name.Length, Type(type));
    }

    internal void TypeSyntax(SourceUnit unit, TypeSyntax syntax, TypeSymbol type)
    {
        if (syntax.SpanEnd <= syntax.Position) return;
        surfaceTypes[(unit.Source.Path, syntax.Position, syntax.SpanEnd)] = Type(type);
        Typed(unit.Source, syntax.Position, syntax.SpanEnd - syntax.Position, Type(type));
        if (syntax.Segments.Count == 1 && syntax.Element is null && syntax.CallableParameters is null &&
            !occurrences.Keys.Any(key => key.Path == unit.Source.Path && key.Offset == syntax.Position))
            Occurrence(unit.Source, syntax.Segments[0].Name, TypeSymbolEntry(type), "type");
    }

    internal void AliasUse(SourceText source, int position, AliasSymbol alias)
    {
        if (TokenAt(source, position) is { } name)
        {
            Occurrence(source, name, Entry(alias, "alias", alias.Syntax.Name.Text), "type");
            if (alias.Target is not null) Typed(source, name.Position, name.Length, Type(alias.Target));
        }
    }

    internal void MetadataAliasUse(SourceText source, Token name, TypeSymbol owner, TypeSymbol target)
    {
        var symbol = External(Metadata(owner, "alias", name.Text), "alias", name.Text);
        symbol.Type ??= Type(target);
        Occurrence(source, name, symbol, "type"); Typed(source, name.Position, name.Length, Type(target));
    }

    internal void MetadataAliasUse(SourceText source, int position, TypeSymbol owner, TypeSymbol target)
    {
        if (TokenAt(source, position) is { } name) MetadataAliasUse(source, name, owner, target);
    }
}
