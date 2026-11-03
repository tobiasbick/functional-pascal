using Fpas.Compiler.Binding.Types;
using System.Reflection;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    private readonly Dictionary<(TypeSymbol Owner, string Name), TypeSymbol?> aliases = [];

    internal TypeSymbol? Alias(TypeSymbol owner, string name)
    {
        if (aliases.TryGetValue((owner, name), out var cached)) return cached;
        var definition = Definition(owner); var reader = definition.Assembly.Reader;
        TypeSymbol? result = null;
        foreach (var handle in definition.Definition.GetMethods())
        {
            var method = reader.GetMethodDefinition(handle);
            var marker = MetadataAttributes.Contract(reader, method.GetCustomAttributes(), "AliasAttribute", [0x20, 2, 1, 8, 14]);
            if (marker is null) continue;
            var blob = marker.Value;
            if (blob.ReadInt32() != 1) throw new BadImageFormatException("Unknown FPAS alias metadata version.");
            var alias = blob.ReadSerializedString();
            if (alias is null || blob.ReadUInt16() != 0 || blob.RemainingBytes != 0 || reader.GetString(method.Name) != "<Alias:" + alias + ">" ||
                !method.Attributes.HasFlag(MethodAttributes.Static) || (method.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Private || method.GetGenericParameters().Count != 0)
                throw new BadImageFormatException("Invalid FPAS alias metadata.");
            if (alias != name) continue;
            var imported = ReadMethod(owner, handle, includePrivate: true);
            if (result is not null || imported is not { Parameters.Count: 0 } || imported.Result.Kind == TypeKind.Void || !PublicType(imported.Result))
                throw new BadImageFormatException("Invalid FPAS alias target signature.");
            result = imported.Result;
        }
        aliases.Add((owner, name), result); return result;
    }
}
