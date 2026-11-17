using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;
using Fpas.Debugging.Values;

namespace Fpas.Debugging.Symbols;

internal sealed class StoredTypeSignature(MetadataImage image, long module, DebugTypeIdentity[] arguments, DebugTypeIdentity[]? methodArguments = null)
{
    private int remaining = 128;

    internal DebugTypeIdentity Field(uint field)
    {
        var definition = image.Metadata.GetFieldDefinition((FieldDefinitionHandle)MetadataTokens.Handle((int)field));
        var blob = image.Metadata.GetBlobReader(definition.Signature);
        if (blob.ReadByte() != 6) throw new BadImageFormatException("Invalid stored-field signature.");
        var type = Read(ref blob, 0);
        if (blob.RemainingBytes != 0) throw new BadImageFormatException("Trailing stored-field signature data.");
        return type;
    }

    internal DebugTypeIdentity? BaseType(uint type)
    {
        var definition = image.Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)type));
        return definition.BaseType.IsNil ? null : Named(definition.BaseType, 18, 0);
    }

    internal DebugTypeIdentity Local(uint method, int slot)
    {
        var blob = image.LocalSignature(method);
        if (blob.ReadByte() != 7) throw new BadImageFormatException("Invalid local signature.");
        var count = blob.ReadCompressedInteger();
        if (count is < 0 or > 4096 || slot < 0 || slot >= count) throw new BadImageFormatException("Invalid local slot.");
        for (var index = 0; index < slot; index++) { remaining = 128; _ = Read(ref blob, 0); }
        remaining = 128;
        return Read(ref blob, 0);
    }

    internal DebugTypeIdentity Argument(uint method, int index)
    {
        var definition = image.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)method));
        var blob = image.Metadata.GetBlobReader(definition.Signature); var header = blob.ReadSignatureHeader();
        if (header.IsGeneric) _ = blob.ReadCompressedInteger();
        var count = blob.ReadCompressedInteger();
        if (count is < 0 or > 4096 || index < 0 || index >= count) throw new BadImageFormatException("Invalid argument index.");
        _ = Read(ref blob, 0);
        for (var position = 0; position < index; position++) { remaining = 128; _ = Read(ref blob, 0); }
        remaining = 128;
        return Read(ref blob, 0);
    }

    private DebugTypeIdentity Read(ref BlobReader blob, int depth)
    {
        if (depth >= 16 || --remaining < 0) throw new DebugException("limit_exceeded", "Stored field types exceed 16 levels or 128 nodes.");
        var kind = blob.ReadByte();
        if (kind == 1) return new("void", kind);
        if (ScalarValue.TypeName(kind) is { } primitive) return new(primitive, kind);
        if (kind is 0x11 or 0x12) return Named(blob.ReadTypeHandle(), kind, depth);
        if (kind is 0x13 or 0x1e)
        {
            var index = blob.ReadCompressedInteger();
            var parameters = kind == 0x13 ? arguments : methodArguments ?? [];
            if (index < 0 || index >= parameters.Length) throw new DebugException("unavailable", "The declared generic argument is unavailable.");
            return parameters[index];
        }
        if (kind == 0x15)
        {
            var category = blob.ReadByte();
            if (category is not (0x11 or 0x12)) throw new BadImageFormatException("Invalid generic field category.");
            var owner = Named(blob.ReadTypeHandle(), category, depth + 1); var count = blob.ReadCompressedInteger();
            if (count is < 1 or > 64) throw new DebugException("limit_exceeded", "Stored field generic arguments exceed 1..64.");
            var parameters = new DebugTypeIdentity[count];
            for (var index = 0; index < count; index++) parameters[index] = Read(ref blob, depth + 1);
            return owner with { Name = Bounded(owner.Name + "<" + string.Join(',', parameters.Select(parameter => parameter.Name)) + ">"), Arguments = parameters };
        }
        if (kind is 0x0f or 0x10 or 0x1d or 0x14)
        {
            var element = Read(ref blob, depth + 1); var rank = 1;
            if (kind == 0x14)
            {
                rank = blob.ReadCompressedInteger();
                if (rank is < 1 or > 32) throw new BadImageFormatException("Invalid stored array rank.");
                var sizes = blob.ReadCompressedInteger();
                if (sizes < 0 || sizes > rank) throw new BadImageFormatException("Invalid stored array sizes.");
                for (var index = 0; index < sizes; index++) _ = blob.ReadCompressedInteger();
                var bounds = blob.ReadCompressedInteger();
                if (bounds < 0 || bounds > rank) throw new BadImageFormatException("Invalid stored array lower bounds.");
                for (var index = 0; index < bounds; index++) _ = blob.ReadCompressedSignedInteger();
            }
            var suffix = kind switch { 0x0f => "*", 0x10 => "&", 0x1d => "[]", _ => "[" + new string(',', rank - 1) + "]" };
            return new(Bounded(element.Name + suffix), kind, Arguments: [element], Rank: kind is 0x14 or 0x1d ? rank : null);
        }
        throw new DebugException("unsupported_expression", "This stored CLR field signature is outside passive type inspection.");
    }

    private DebugTypeIdentity Named(EntityHandle handle, uint kind, int depth)
    {
        if (depth >= 16 || --remaining < 0) throw new DebugException("limit_exceeded", "Stored field types exceed 16 levels or 128 nodes.");
        if (handle.Kind == HandleKind.TypeDefinition)
        {
            var token = (uint)MetadataTokens.GetToken(handle);
            return new(image.TypeName(token), kind, module, image.Identity, token);
        }
        if (handle.Kind == HandleKind.TypeReference) return new(ReferenceName((TypeReferenceHandle)handle, 0), kind, Exact: false);
        if (handle.Kind == HandleKind.TypeSpecification)
        {
            var blob = image.Metadata.GetBlobReader(image.Metadata.GetTypeSpecification((TypeSpecificationHandle)handle).Signature);
            var value = Read(ref blob, depth + 1);
            if (blob.RemainingBytes != 0) throw new BadImageFormatException("Trailing stored type specification data.");
            return value;
        }
        throw new BadImageFormatException("Invalid stored named type.");
    }

    private string ReferenceName(TypeReferenceHandle handle, int depth)
    {
        if (depth >= 16 || --remaining < 0) throw new DebugException("limit_exceeded", "Stored field type references exceed their depth/node limit.");
        var type = image.Metadata.GetTypeReference(handle); var name = image.Metadata.GetString(type.Name); var ns = image.Metadata.GetString(type.Namespace);
        return Bounded(type.ResolutionScope.Kind == HandleKind.TypeReference ? ReferenceName((TypeReferenceHandle)type.ResolutionScope, depth + 1) + "+" + name : ns.Length == 0 ? name : ns + "." + name);
    }
    private static string Bounded(string name) => name.Length <= 8192 ? name : throw new DebugException("limit_exceeded", "A stored field type name exceeds 8192 UTF-16 code units.");
}
