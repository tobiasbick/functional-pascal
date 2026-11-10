using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal static IReadOnlyList<GenericParameterSymbol> SourceParameters(TypeSymbol type)
        => type.Interface?.GenericParameters ?? type.Record?.GenericParameters ?? type.Variant?.GenericParameters ?? type.Closure?.GenericParameters ?? [];

    internal ImportedField RecordField(RecordFieldSymbol field, TypeSymbol owner) => new(owner, "<" + field.Name + ">k__BackingField",
        field.Type, false, true, SignatureType: SourceSignature(field.Type, SourceParameters(owner)));

    internal TypeSymbol ConstructSource(TypeSymbol definition, IReadOnlyList<TypeSymbol> arguments, SourceText source, int position, bool validate = true)
    {
        var parameters = SourceParameters(definition);
        if (parameters.Count != arguments.Count)
            throw source.Error("FP3026", "A source type requires the exact number of explicit type arguments.", position);
        if (validate && !ValidSourceArguments(parameters, arguments))
            throw source.Error("FP3018", "Type arguments violate source type constraints.", position);
        return definition with { Arguments = arguments.Count == 0 ? null : new(arguments) };
    }

    internal RecordInfo SourceRecordInfo(TypeSymbol type)
    {
        var info = type.Record!.Info;
        TypeSymbol Apply(TypeSymbol field) => SubstituteSource(field, SourceParameters(type), type.TypeArguments);
        ImportedMethod Method(ImportedMethod method) => InstantiateSource(method.SourceMethod!, type, null);
        return new(type, info.Fields.Select(field => new RecordFieldInfo(field.Name, Apply(field.Type), field.IsPublic,
            Method(field.Getter), Method(field.Copy), field.Default is null ? null : Method(field.Default))).ToArray(), Method(info.Constructor), info.CanConstruct);
    }

    internal VariantInfo SourceVariantInfo(TypeSymbol type)
    {
        var variant = type.Variant!;
        return new(type, variant.Cases.Select(member =>
        {
            var representation = member.Representation.Type with { Arguments = type.Arguments };
            return new VariantCaseInfo(member.Syntax.Name.Text, member.Tag, representation, InstantiateSource(member.Factory, type, null),
                member.Representation.Fields.Select(field => new VariantPayload(field.Name, SubstituteSource(field.Type, variant.GenericParameters, type.TypeArguments),
                    InstantiateSource(field.Getter, representation, null))).ToArray());
        }).ToArray());
    }

    internal TypeSymbol SourceSignature(TypeSymbol type, IReadOnlyList<GenericParameterSymbol> owner)
    {
        if (type.Kind == TypeKind.Generated && type.Arguments is null && type.Closure!.GenericParameters.Count != 0)
            type = type with { Arguments = new(type.Closure.GenericParameters.Select(parameter => parameter.Type)) };
        if (type.IsParameter)
        {
            var slot = owner.ToList().IndexOf(type.Parameter!);
            var parameter = new TypeSymbol(slot >= 0 ? TypeKind.TypeParameter : TypeKind.MethodParameter, ParameterIndex: slot >= 0 ? slot : type.Parameter!.Index);
            if (type.NullableValue) return Symbol(Required("System.Nullable`1")) with { Arguments = new([parameter]) };
            return parameter;
        }
        return type with { Element = type.Element is null ? null : SourceSignature(type.Element, owner),
            Arguments = type.Arguments is null ? null : new(type.Arguments.Select(argument => SourceSignature(argument, owner))) };
    }
}
