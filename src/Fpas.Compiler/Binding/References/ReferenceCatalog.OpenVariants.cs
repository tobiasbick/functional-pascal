using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;

namespace Fpas.Compiler.Binding.References;

internal sealed record OpenVariantInfo(VariantInfo Info, TypeList Variables);

internal sealed partial class ReferenceCatalog
{
    private readonly Dictionary<TypeSymbol, OpenVariantInfo?> openVariants = [];

    internal OpenVariantInfo? OpenVariant(TypeSymbol type)
    {
        if (openVariants.TryGetValue(type, out var existing)) return existing;
        if (type.Variant is { GenericParameters.Count: > 0 } variant)
        {
            var source = new OpenVariantInfo(SourceVariantInfo(type), new(variant.GenericParameters.Select(parameter => parameter.Type)));
            openVariants.Add(type, source); return source;
        }
        if (type.Definition is not { } definition || definition.Definition.GetGenericParameters().Count == 0 ||
            MetadataAttributes.Contract(definition.Assembly.Reader, definition.Definition.GetCustomAttributes(), "VariantAttribute", [0x20, 1, 1, 8]) is null)
        { openVariants.Add(type, null); return null; }
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
        var imported = new OpenVariantInfo(VariantInfo(type with { Arguments = variables })!, variables);
        openVariants.Add(type, imported); return imported;
    }

    internal TypeSymbol CloseVariant(OpenVariantInfo open, TypeList arguments, Syntax.SourceText source, int position)
        => open.Info.Type.Variant is not null ? ConstructSource(open.Info.Type, arguments, source, position) : Construct(open.Info.Type.Definition!, arguments, source, position);
}
