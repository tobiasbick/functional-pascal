using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    private bool OverridesInterfaceMember(TypeSymbol owner, EntityHandle declaration, ImportedMethod target)
    {
        var definition = Definition(owner); var reader = definition.Assembly.Reader;
        if (declaration.Kind == HandleKind.MethodDefinition)
            return target.Owner.Definition?.Assembly == definition.Assembly && target.DefinitionHandle == (MethodDefinitionHandle)declaration;
        if (declaration.Kind != HandleKind.MemberReference) return false;
        var member = reader.GetMemberReference((MemberReferenceHandle)declaration);
        if (reader.GetString(member.Name) != target.Name || Decode(definition, member.Parent, owner).Erased != target.Owner.Erased) return false;
        var signature = member.DecodeMethodSignature(new SignatureTypes(this, definition.Assembly), null);
        return signature.GenericParameterCount == (target.MethodArguments?.Count ?? 0) &&
            Substitute(signature.ReturnType, target.Owner, target.MethodArguments, keepMethodParameters: true).Erased == target.Result.Erased &&
            signature.ParameterTypes.Select(type => Substitute(type.Kind == TypeKind.ByRef ? type.Element! : type, target.Owner, target.MethodArguments, keepMethodParameters: true).Erased)
                .SequenceEqual(target.Parameters.Select(type => type.Erased));
    }
}
