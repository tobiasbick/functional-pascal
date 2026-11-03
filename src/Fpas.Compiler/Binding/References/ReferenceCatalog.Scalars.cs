using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Globalization;
using System.Numerics;
using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    private readonly Dictionary<TypeSymbol, ScalarInfo?> scalarTypes = [];

    internal ScalarInfo? ScalarInfo(TypeSymbol type)
    {
        type = type.NonNullable;
        if (type.Scalar is { } source)
        {
            source.EnsureBounds();
            return new(type, source.Base, source.Syntax.Category, source.Lower, source.Upper);
        }
        if (type.Kind != TypeKind.Named) return null;
        if (scalarTypes.TryGetValue(type, out var cached)) return cached;
        var definition = Definition(type); var reader = definition.Assembly.Reader;
        if (definition.IsEnum)
        {
            var field = definition.Definition.GetFields().Select(reader.GetFieldDefinition).Single(field => reader.GetString(field.Name) == "value__");
            var primitive = field.DecodeSignature(new SignatureTypes(this, definition.Assembly), null);
            var info = new ScalarInfo(type, primitive, ScalarCategory.Enum); scalarTypes.Add(type, info); return info;
        }
        var marker = MetadataAttributes.Contract(reader, definition.Definition.GetCustomAttributes(), "DomainAttribute", [0x20, 4, 1, 8, 14, 14, 14]);
        if (marker is null) { scalarTypes.Add(type, null); return null; }
        var blob = marker.Value;
        if (blob.ReadInt32() != 1) throw InvalidScalar(type);
        var baseName = blob.ReadSerializedString(); var lowerText = blob.ReadSerializedString(); var upperText = blob.ReadSerializedString();
        var baseType = baseName is null ? TypeSymbol.Unsupported : TypeFacts.Primitive(baseName);
        if (blob.ReadUInt16() != 0 || blob.RemainingBytes != 0 || lowerText is null != (upperText is null) ||
            !(TypeFacts.Numeric(baseType) || baseType == TypeSymbol.String || baseType == TypeSymbol.Boolean) ||
            definition.Definition.GetGenericParameters().Count != 0 || !definition.Definition.Attributes.HasFlag(TypeAttributes.Sealed) ||
            type.IsValue == (baseType == TypeSymbol.String) || Decode(definition, definition.Definition.BaseType, type) !=
                (baseType == TypeSymbol.String ? TypeSymbol.Object : Symbol(Required("System.ValueType")))) throw InvalidScalar(type);
        BigInteger? lower = null, upper = null;
        if (lowerText is not null)
        {
            if (!NumericFacts.Integral(baseType) || baseType.Kind is TypeKind.Char or TypeKind.NativeInt or TypeKind.NativeUInt ||
                !BigInteger.TryParse(lowerText, CultureInfo.InvariantCulture, out var lo) || !BigInteger.TryParse(upperText, CultureInfo.InvariantCulture, out var hi) ||
                lo > hi || !NumericFacts.Fits(lo, baseType) || !NumericFacts.Fits(hi, baseType)) throw InvalidScalar(type);
            lower = lo; upper = hi;
        }
        var result = new ScalarInfo(type, baseType, lower is null ? ScalarCategory.Distinct : ScalarCategory.Range, lower, upper);
        var fields = definition.Definition.GetFields().Select(reader.GetFieldDefinition).Where(field => !field.Attributes.HasFlag(FieldAttributes.Static)).ToArray();
        if (fields.Length != 1 || fields[0].Attributes != (FieldAttributes.Private | FieldAttributes.InitOnly) ||
            fields[0].DecodeSignature(new SignatureTypes(this, definition.Assembly), null) != (lower is null ? baseType : TypeSymbol.UInt64)) throw InvalidScalar(type);
        var property = Property(type, "Value", false);
        var values = definition.Definition.GetProperties().Select(reader.GetPropertyDefinition).Where(value => reader.GetString(value.Name) == "Value").ToArray();
        if (values.Length != 1 || !values[0].GetAccessors().Setter.IsNil || property?.Type != baseType ||
            property.Getter is not { Name: "get_Value", Parameters.Count: 0 } || property.Getter.Result != baseType) throw InvalidScalar(type);
        _ = RecordHelper(type, ".ctor", false, [baseType], TypeSymbol.Void, MethodAttributes.Public);
        _ = RecordHelper(type, "op_Explicit", true, [baseType], type, MethodAttributes.Public);
        _ = RecordHelper(type, lower is null ? "op_Explicit" : "op_Implicit", true, [type], baseType, MethodAttributes.Public);
        var nullable = type.IsReference ? type with { IsNullable = true } : type;
        foreach (var name in new[] { "op_Equality", "op_Inequality" })
            _ = RecordHelper(type, name, true, [nullable, nullable], TypeSymbol.Boolean, MethodAttributes.Public);
        _ = RecordHelper(type, "Equals", false, [nullable], TypeSymbol.Boolean, MethodAttributes.Public);
        _ = RecordHelper(type, "Equals", false, [TypeSymbol.Object with { IsNullable = true }], TypeSymbol.Boolean, MethodAttributes.Public);
        _ = RecordHelper(type, "GetHashCode", false, [], TypeSymbol.Int32, MethodAttributes.Public);
        if (!Parents(type).Contains(Symbol(Required("System.IEquatable`1")) with { Arguments = new([type]) })) throw InvalidScalar(type);
        if (TypeFacts.Numeric(baseType) || baseType == TypeSymbol.String)
            foreach (var name in new[] { "op_LessThan", "op_GreaterThan", "op_LessThanOrEqual", "op_GreaterThanOrEqual" })
                _ = RecordHelper(type, name, true, [type, type], TypeSymbol.Boolean, MethodAttributes.Public);
        if (lower is not null)
            foreach (var (name, expected) in new[] { ("LowerBound", lower.Value), ("UpperBound", upper!.Value) })
            {
                var bound = Field(type, name, true);
                if (bound?.IsLiteral != true || bound.Type != baseType || NumericValue.Integer(bound.Constant!) != expected) throw InvalidScalar(type);
            }
        scalarTypes.Add(type, result); return result;
    }

    internal static ImportedMethod DomainGetter(ScalarInfo info) => new(info.Type, "get_Value", info.Base, [], false);
    internal static ImportedMethod DomainConstructor(ScalarInfo info) => new(info.Type, ".ctor", TypeSymbol.Void, [info.Base], false);

    private static BadImageFormatException InvalidScalar(TypeSymbol type) => new($"Invalid or unsupported FPAS scalar metadata on '{type}'.");

    private IReadOnlyList<ImportedMethod> ScalarMethods(TypeSymbol type)
    {
        var info = ScalarInfo(type)!;
        if (info.Category == ScalarCategory.Enum) return [];
        var nullable = type.IsReference ? type with { IsNullable = true } : type;
        return [DomainConstructor(info), DomainGetter(info), new(type, "Equals", TypeSymbol.Boolean, [nullable], false, true),
            new(type, "Equals", TypeSymbol.Boolean, [TypeSymbol.Object with { IsNullable = true }], false, true), new(type, "GetHashCode", TypeSymbol.Int32, [], false, true),
            new(type, "op_Equality", TypeSymbol.Boolean, [nullable, nullable], true), new(type, "op_Inequality", TypeSymbol.Boolean, [nullable, nullable], true)];
    }
}
