using System.Reflection;

namespace Fpas.Compiler.Binding;

internal sealed record OpenUnionInfo(UnionInfo Info, TypeList Variables);

internal sealed partial class ReferenceCatalog
{
    private readonly Dictionary<TypeSymbol, OpenUnionInfo?> openUnions = [];

    internal OpenUnionInfo? OpenUnion(TypeSymbol type)
    {
        if (openUnions.TryGetValue(type, out var existing)) return existing;
        if (type.Union is { GenericParameters.Count: > 0 } union)
        {
            var source = new OpenUnionInfo(SourceUnionInfo(type), new(union.GenericParameters.Select(parameter => parameter.Type)));
            openUnions.Add(type, source); return source;
        }
        if (type.Definition is not { } definition || definition.Definition.GetGenericParameters().Count == 0 ||
            MetadataAttributes.Contract(definition.Assembly.Reader, definition.Definition.GetCustomAttributes(), "UnionAttribute", [0x20, 1, 1, 8]) is null)
        { openUnions.Add(type, null); return null; }
        var variables = new TypeList(definition.Definition.GetGenericParameters().Select(handle =>
        {
            var parameter = definition.Assembly.Reader.GetGenericParameter(handle);
            return new GenericParameterSymbol(definition.Assembly.Reader.GetString(parameter.Name), parameter.Index)
            {
                Primary = parameter.Attributes.HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint) ? "struct" : null,
                IsReference = parameter.Attributes.HasFlag(GenericParameterAttributes.ReferenceTypeConstraint),
                IsNotNull = NullabilityMetadata.GenericNotNull(definition, parameter, null)
            }.Type;
        }));
        var imported = new OpenUnionInfo(UnionInfo(type with { Arguments = variables })!, variables);
        openUnions.Add(type, imported); return imported;
    }

    internal TypeSymbol CloseUnion(OpenUnionInfo open, TypeList arguments, Syntax.SourceText source, int position)
        => open.Info.Type.Union is not null ? ConstructSource(open.Info.Type, arguments, source, position) : Construct(open.Info.Type.Definition!, arguments, source, position);
}
