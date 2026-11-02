using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal enum GeneratedMethodKind { None, Constructor, Getter, Copy, Default, EqualsTyped, EqualsObject, Hash, Equality, Inequality, Tag, Factory }

internal sealed class RecordSymbol(SourceUnit owner, RecordSyntax syntax)
{
    internal SourceUnit Owner { get; } = owner;
    internal RecordSyntax Syntax { get; } = syntax;
    internal string Name => Syntax.Name.Text;
    internal UnionCaseSymbol? UnionCase { get; set; }
    internal bool IsPublic => Syntax.IsPublic;
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; set; } = [];
    internal TypeSymbol Type => new(TypeKind.Record, Record: this, Arguments: GenericParameters.Count == 0 ? null : new(GenericParameters.Select(parameter => parameter.Type)));
    internal List<RecordFieldSymbol> Fields { get; } = [];
    internal List<RoutineSymbol> Methods { get; } = [];
    internal Dictionary<string, List<RoutineSymbol>> UserMethods { get; } = new(StringComparer.Ordinal);
    internal Dictionary<RoutineSymbol, BoundRoutine> Bodies { get; } = [];
    internal List<TypeSymbol> Interfaces { get; } = [];
    internal List<(ImportedMethod Contract, RoutineSymbol Body)> InterfaceMethods { get; } = [];
    internal RecordInfo Info { get; set; } = null!;
    internal RoutineSymbol Constructor => Methods.Single(method => method.Generated == GeneratedMethodKind.Constructor);
    internal ImportedMethod Method(RoutineSymbol method) => ReferenceCatalog.SourceMethod(method, Type);
}

internal sealed class RecordFieldSymbol(RecordFieldSyntax syntax, TypeSymbol type)
{
    internal RecordFieldSyntax Syntax { get; } = syntax;
    internal string Name => Syntax.Name.Text;
    internal UnionCaseSymbol? UnionCase { get; set; }
    internal TypeSymbol Type { get; } = type;
    internal RoutineSymbol Getter { get; set; } = null!;
    internal RoutineSymbol? Copy { get; set; }
    internal RoutineSymbol? Default { get; set; }
}

internal sealed record RecordFieldInfo(string Name, TypeSymbol Type, bool IsPublic, ImportedMethod Getter, ImportedMethod Copy, ImportedMethod? Default);
internal sealed record RecordInfo(TypeSymbol Type, IReadOnlyList<RecordFieldInfo> Fields, ImportedMethod Constructor, bool CanConstruct);
