using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    private static IReadOnlyList<ParameterSymbol> ReadParameters(ImportedType owner, MethodDefinition method,
        IReadOnlyList<TypeSymbol> types, IReadOnlyList<TypeSymbol> signatures)
    {
        var reader = owner.Assembly.Reader;
        var result = types.Select((type, index) => new ParameterSymbol("", type, index, signatures[index].Kind == TypeKind.ByRef ? "ref" : "")).ToArray();
        foreach (var handle in method.GetParameters())
        {
            var parameter = reader.GetParameter(handle);
            if (parameter.SequenceNumber == 0) continue;
            var index = parameter.SequenceNumber - 1;
            var mode = signatures[index].Kind == TypeKind.ByRef
                ? parameter.Attributes.HasFlag(ParameterAttributes.Out) && !parameter.Attributes.HasFlag(ParameterAttributes.In) ? "out"
                    : parameter.Attributes.HasFlag(ParameterAttributes.In) || MetadataAttributes.Has(reader, parameter.GetCustomAttributes(), "System.Runtime.CompilerServices.IsReadOnlyAttribute") ? "in" : "ref"
                : MetadataAttributes.Has(reader, parameter.GetCustomAttributes(), "System.ParamArrayAttribute") ? "params" : "";
            result[index] = new(reader.GetString(parameter.Name), types[index], index, mode);
            if (parameter.Attributes.HasFlag(ParameterAttributes.Optional) && !parameter.GetDefaultValue().IsNil)
            {
                var constant = reader.GetConstant(parameter.GetDefaultValue());
                var blob = reader.GetBlobReader(constant.Value);
                result[index].Default = constant.TypeCode switch
                {
                    ConstantTypeCode.Int32 when types[index].Definition?.IsEnum == true => new BoundField(new ImportedField(types[index], "<default>", types[index], true, true, blob.ReadInt32(), true), null),
                    ConstantTypeCode.Int32 => new BoundInteger(blob.ReadInt32()),
                    ConstantTypeCode.Double => new BoundDouble(blob.ReadDouble()),
                    ConstantTypeCode.Boolean => new BoundBoolean(blob.ReadBoolean()),
                    ConstantTypeCode.Char => new BoundCharacter((char)blob.ReadUInt16()),
                    ConstantTypeCode.String => new BoundString(blob.ReadUTF16(blob.Length)),
                    ConstantTypeCode.NullReference => types[index].IsValue && !types[index].NullableValue ? new BoundNew(types[index], null, []) : new BoundNull(),
                    _ => null
                };
            }
        }
        return result;
    }

    internal ImportedMethod CloseMethod(ImportedMethod method, TypeList arguments)
        => method.SourceMethod is { } source ? InstantiateSource(source, method.Owner, arguments)
            : ReadMethod(method.Owner, method.DefinitionHandle, arguments)!;
}
