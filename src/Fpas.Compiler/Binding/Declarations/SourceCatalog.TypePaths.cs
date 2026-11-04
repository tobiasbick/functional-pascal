using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    internal ImportTarget ResolveTypePath(IReadOnlyList<TypeSegmentSyntax> segments, SourceUnit unit, ReferenceCatalog references,
        IReadOnlyDictionary<string, GenericParameterSymbol>? generic = null, bool validate = true, int inferredArity = 0, bool allowOpenGeneric = false)
    {
        ImportTarget? target = null;
        for (var index = 0; index < segments.Count; index++)
        {
            var segment = segments[index];
            var arguments = segment.Arguments?.Select(argument => ResolveType(argument, unit, references, generic, validate)).ToArray();
            var last = index == segments.Count - 1;
            if (target is null or NamespaceImport or SourceImport)
            {
                target = ResolvePath(segments.Take(index + 1).Select(part => part.Name.Text).ToArray(), unit, references,
                    segment.Name.Position, arguments?.Length ?? (last ? inferredArity : 0), last && allowOpenGeneric);
                if (arguments is not null) target = new ClrTypeImport(ConstructTarget(target, arguments, segment.Name.Position));
            }
            else
            {
                var owner = target switch
                {
                    ClrTypeImport closed => closed.Type,
                    ClrImport imported => references.Symbol(imported.Type),
                    RecordImport record => record.Record.Type,
                    UnionImport union => union.Union.Type,
                    InterfaceImport contract => contract.Interface.Type,
                    _ => TypeSymbol.Unsupported
                };
                if (owner.Definition is { } definition && definition.Definition.GetGenericParameters().Count != owner.TypeArguments.Count ||
                    target is RecordImport { Record.GenericParameters.Count: > 0 } or UnionImport { Union.GenericParameters.Count: > 0 } or InterfaceImport { Interface.GenericParameters.Count: > 0 })
                    throw unit.Source.Error("FP3026", "An enclosing generic type requires explicit type arguments.", segments[index - 1].Name.Position);
                if (owner.Union is { } unionOwner)
                {
                    if (arguments is not null) throw unit.Source.Error("FP3018", "A union case type inherits its enclosing type arguments.", segment.Name.Position);
                    target = new ClrTypeImport(SourceCaseType(unionOwner, segment.Name.Text, segment.Name.Position, unit.Source).Type with { Arguments = owner.Arguments });
                }
                else if (owner.Definition is not null)
                {
                    if (arguments is null && references.Alias(owner, segment.Name.Text) is { } alias)
                    {
                        Analysis?.MetadataAliasUse(unit.Source, segment.Name, owner, alias);
                        target = new ClrTypeImport(alias);
                    }
                    else target = references.NestedType(owner, segment.Name, arguments, unit.Source, validate,
                        last ? inferredArity : 0, last && allowOpenGeneric);
                }
                else throw unit.Source.Error("FP3005", $"Type '{owner}' has no nested type '{segment.Name.Text}'.", segment.Name.Position);
            }
            Analysis?.TypePart(unit, segment.Name, target, index == 0);
        }
        return target!;

        TypeSymbol ConstructTarget(ImportTarget selected, IReadOnlyList<TypeSymbol> arguments, int position) => selected switch
        {
            RecordImport own => references.ConstructSource(own.Record.Type, arguments, unit.Source, position, validate),
            UnionImport own => references.ConstructSource(own.Union.Type, arguments, unit.Source, position, validate),
            InterfaceImport own => references.ConstructSource(own.Interface.Type, arguments, unit.Source, position, validate),
            ClrTypeImport own when own.Type.Record is not null && own.Type.TypeArguments.Count == 0 => references.ConstructSource(own.Type, arguments, unit.Source, position, validate),
            ClrImport imported => references.Construct(imported.Type, arguments, unit.Source, position, validate),
            _ => throw unit.Source.Error("FP3018", "Type arguments require a generic type definition.", position)
        };
    }
}
