using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding.References;

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
            if (parameter.Attributes.HasFlag(ParameterAttributes.Optional) && MetadataConstants.Decimal(reader, parameter.GetCustomAttributes()) is { } decimalValue)
                result[index].Default = new BoundNumber(TypeSymbol.Decimal, decimalValue);
            else if (parameter.Attributes.HasFlag(ParameterAttributes.Optional) && !parameter.GetDefaultValue().IsNil)
            {
                var value = MetadataConstants.Read(reader, parameter.GetDefaultValue());
                result[index].Default = value switch
                {
                    bool boolean => new BoundBoolean(boolean), string text => new BoundString(text),
                    null => types[index].IsValue && !types[index].NullableValue ? new BoundNew(types[index], null, []) : new BoundNull(),
                    _ => BoundNumber.Create(types[index].NonNullable, value)
                };
            }
        }
        return result;
    }

    internal ImportedMethod CloseMethod(ImportedMethod method, TypeList arguments)
        => method.SourceMethod is { } source ? InstantiateSource(source, method.Owner, arguments)
            : ReadMethod(method.Owner, method.DefinitionHandle, arguments)! with { ConstrainedType = method.ConstrainedType };
}
