using System.Collections.Immutable;
using System.Reflection.Metadata;
using System.Text.RegularExpressions;

namespace Fpas.Workspaces.Documentation;

internal sealed record MetadataType(string Display, string Xml);
internal sealed record GenericNames(string[] Type, string[] Method);

internal sealed class MetadataSignatureTypes : ISignatureTypeProvider<MetadataType, GenericNames>
{
    internal static string DefinitionName(MetadataReader reader, TypeDefinitionHandle handle)
    {
        var type = reader.GetTypeDefinition(handle); var name = reader.GetString(type.Name);
        return !type.GetDeclaringType().IsNil ? DefinitionName(reader, type.GetDeclaringType()) + "." + name :
            reader.GetString(type.Namespace) is { Length: > 0 } ns ? ns + "." + name : name;
    }

    private static string ReferenceName(MetadataReader reader, TypeReferenceHandle handle)
    {
        var type = reader.GetTypeReference(handle); var name = reader.GetString(type.Name);
        return type.ResolutionScope.Kind == HandleKind.TypeReference ? ReferenceName(reader, (TypeReferenceHandle)type.ResolutionScope) + "." + name :
            reader.GetString(type.Namespace) is { Length: > 0 } ns ? ns + "." + name : name;
    }

    internal static string WithoutArity(string name) => Regex.Replace(name, @"`\d+", "", RegexOptions.CultureInvariant);
    public MetadataType GetTypeFromDefinition(MetadataReader reader, TypeDefinitionHandle handle, byte kind) => Named(DefinitionName(reader, handle));
    public MetadataType GetTypeFromReference(MetadataReader reader, TypeReferenceHandle handle, byte kind) => Named(ReferenceName(reader, handle));
    public MetadataType GetTypeFromSpecification(MetadataReader reader, GenericNames context, TypeSpecificationHandle handle, byte kind)
        => reader.GetTypeSpecification(handle).DecodeSignature(this, context);
    private static MetadataType Named(string name) => new(WithoutArity(name), name);
    public MetadataType GetGenericTypeParameter(GenericNames context, int index) => new(index < context.Type.Length ? context.Type[index] : "T" + index, "`" + index);
    public MetadataType GetGenericMethodParameter(GenericNames context, int index) => new(index < context.Method.Length ? context.Method[index] : "M" + index, "``" + index);
    public MetadataType GetGenericInstantiation(MetadataType type, ImmutableArray<MetadataType> arguments)
    {
        var index = 0;
        string Instantiate(bool xml)
        {
            index = 0;
            return Regex.Replace(type.Xml, @"`(\d+)", match =>
            {
                var count = int.Parse(match.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture);
                var parts = arguments.Skip(index).Take(count).Select(argument => xml ? argument.Xml : argument.Display); index += count;
                return (xml ? "{" : "<") + string.Join(xml ? "," : ", ", parts) + (xml ? "}" : ">");
            }, RegexOptions.CultureInvariant);
        }
        return new(Instantiate(false), Instantiate(true));
    }
    public MetadataType GetArrayType(MetadataType type, ArrayShape shape) => new(type.Display + "[" + new string(',', shape.Rank - 1) + "]",
        type.Xml + "[" + string.Join(',', Enumerable.Range(0, shape.Rank).Select(index =>
            (index < shape.LowerBounds.Length ? shape.LowerBounds[index] : 0).ToString(System.Globalization.CultureInfo.InvariantCulture) + ":" +
            (index < shape.Sizes.Length ? shape.Sizes[index].ToString(System.Globalization.CultureInfo.InvariantCulture) : ""))) + "]");
    public MetadataType GetSZArrayType(MetadataType type) => new(type.Display + "[]", type.Xml + "[]");
    public MetadataType GetByReferenceType(MetadataType type) => new(type.Display + "&", type.Xml + "@");
    public MetadataType GetPointerType(MetadataType type) => new(type.Display + "*", type.Xml + "*");
    public MetadataType GetPinnedType(MetadataType type) => type with { Display = type.Display + " pinned" };
    public MetadataType GetModifiedType(MetadataType modifier, MetadataType type, bool required)
        => type with { Display = type.Display + (required ? " modreq(" : " modopt(") + modifier.Display + ")" };
    public MetadataType GetFunctionPointerType(MethodSignature<MetadataType> signature)
        // C# XML documentation leaves the function-pointer parameter position empty.
        => new("methodptr(" + string.Join(", ", signature.ParameterTypes.Select(type => type.Display)) + ") -> " + signature.ReturnType.Display, "");
    public MetadataType GetPrimitiveType(PrimitiveTypeCode code)
    {
        var name = code switch
        {
            PrimitiveTypeCode.Void => "Void", PrimitiveTypeCode.Boolean => "Boolean", PrimitiveTypeCode.Char => "Char",
            PrimitiveTypeCode.SByte => "SByte", PrimitiveTypeCode.Byte => "Byte", PrimitiveTypeCode.Int16 => "Int16",
            PrimitiveTypeCode.UInt16 => "UInt16", PrimitiveTypeCode.Int32 => "Int32", PrimitiveTypeCode.UInt32 => "UInt32",
            PrimitiveTypeCode.Int64 => "Int64", PrimitiveTypeCode.UInt64 => "UInt64", PrimitiveTypeCode.Single => "Single",
            PrimitiveTypeCode.Double => "Double", PrimitiveTypeCode.String => "String", PrimitiveTypeCode.Object => "Object",
            PrimitiveTypeCode.IntPtr => "IntPtr", PrimitiveTypeCode.UIntPtr => "UIntPtr", PrimitiveTypeCode.TypedReference => "TypedReference",
            _ => throw new BadImageFormatException("Unknown metadata primitive.")
        };
        return new("System." + name, "System." + name);
    }
}
