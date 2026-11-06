using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Symbols;

internal sealed partial class ModuleSymbols
{
    internal int? NextBindingOffset(uint token, int offset)
    {
        var method = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        var bytes = Image.GetMethodBody(method.RelativeVirtualAddress).GetILBytes();
        if (bytes is null || offset < 0 || offset >= bytes.Length) return null;
        // Source storage and fallible operations remain barriers; unnamed argument temporaries may be filled.
        var size = bytes[offset] switch
        {
            0x00 or >= 0x02 and <= 0x09 or >= 0x14 and <= 0x1e or 0x25 or 0x26 => 1,
            >= 0x0a and <= 0x0d when !NamedSlot(bytes[offset] - 0x0a) => 1,
            0x13 when offset + 1 < bytes.Length && !NamedSlot(bytes[offset + 1]) => 2,
            0x0e or 0x0f or 0x11 or 0x12 or 0x1f => 2,
            0x20 or 0x22 or 0x72 => 5,
            0x21 or 0x23 => 9,
            0xfe when offset + 1 < bytes.Length && bytes[offset + 1] is 0x09 or 0x0a or 0x0c or 0x0d => 4,
            _ => 0
        };
        var next = offset + size;
        return size > 0 && next < bytes.Length && next < NextOffset(token, offset) ? next : null;

        bool NamedSlot(int slot) => Pdb.GetLocalScopes((MethodDefinitionHandle)MetadataTokens.Handle((int)token))
            .Select(Pdb.GetLocalScope).SelectMany(scope => scope.GetLocalVariables()).Select(Pdb.GetLocalVariable).Any(local => local.Index == slot);
    }

    internal bool HasSourcePoints(uint token) => Pdb.GetMethodDebugInformation((MethodDefinitionHandle)MetadataTokens.Handle((int)token))
        .GetSequencePoints().Any(point => !point.IsHidden);
    internal bool IsSourcePoint(uint token, int offset) => Pdb.GetMethodDebugInformation((MethodDefinitionHandle)MetadataTokens.Handle((int)token))
        .GetSequencePoints().Any(point => !point.IsHidden && point.Offset == offset);
    internal bool HasNextSourcePoint(uint token, int offset) => Pdb.GetMethodDebugInformation((MethodDefinitionHandle)MetadataTokens.Handle((int)token))
        .GetSequencePoints().Any(point => !point.IsHidden && point.Offset > offset);

    internal SourceLocation? Position(uint token, int offset)
    {
        var method = (MethodDefinitionHandle)MetadataTokens.Handle((int)token);
        var information = Pdb.GetMethodDebugInformation(method);
        var points = information.GetSequencePoints().Where(point => point.Offset <= offset).ToArray();
        if (points.Length == 0 || points[^1].IsHidden) return null;
        var point = points[^1];
        var document = point.Document.IsNil ? information.Document : point.Document;
        return new(DocumentPath(document), point.StartLine, point.StartColumn, point.EndLine, point.EndColumn, SourceStatus(document));
    }

    internal SourceLocation? VisibleBefore(uint token, int offset)
    {
        var method = (MethodDefinitionHandle)MetadataTokens.Handle((int)token);
        var points = Pdb.GetMethodDebugInformation(method).GetSequencePoints().Where(point => !point.IsHidden && point.Offset <= offset).ToArray();
        return points.Length == 0 ? null : Position(token, points[^1].Offset);
    }

    internal string MethodName(uint token)
    {
        var definition = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        var type = Metadata.GetTypeDefinition(definition.GetDeclaringType());
        return Metadata.GetString(type.Name) + "." + Metadata.GetString(definition.Name);
    }

    internal (string Name, int Index)[] Arguments(uint token, string view = "raw")
    {
        var definition = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        var instance = (definition.Attributes & MethodAttributes.Static) == 0;
        var values = definition.GetParameters().Select(Metadata.GetParameter).Where(parameter => parameter.SequenceNumber != 0)
            .Select(parameter => (Name: Metadata.GetString(parameter.Name), Index: parameter.SequenceNumber - (instance ? 0 : 1))).ToList();
        if (instance) values.Insert(0, (view == "source" && SourceReceiver(token) ? "self" : "this", 0));
        return values.ToArray();
    }

    internal bool SourceReceiver(uint token)
    {
        var method = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        try { return Shape((uint)MetadataTokens.GetToken(method.GetDeclaringType()))?.Kind is "record" or "union"; }
        catch (Exception exception) when (exception is BadImageFormatException or DebugException) { return false; }
    }

    internal bool Generated(uint token)
    {
        var method = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        return HasHiddenAttribute(method.GetCustomAttributes()) || HasHiddenAttribute(Metadata.GetTypeDefinition(method.GetDeclaringType()).GetCustomAttributes());
    }

    internal bool ShouldSkip(uint token)
    {
        var method = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        return HasHiddenAttribute(method.GetCustomAttributes(), false) || HasHiddenAttribute(Metadata.GetTypeDefinition(method.GetDeclaringType()).GetCustomAttributes(), false);
    }

    private bool HasHiddenAttribute(CustomAttributeHandleCollection handles, bool includeGenerated = true)
    {
        foreach (var handle in handles)
        {
            var attribute = Metadata.GetCustomAttribute(handle);
            var type = attribute.Constructor.Kind == HandleKind.MemberReference
                ? Metadata.GetMemberReference((MemberReferenceHandle)attribute.Constructor).Parent
                : Metadata.GetMethodDefinition((MethodDefinitionHandle)attribute.Constructor).GetDeclaringType();
            if (type.Kind != HandleKind.TypeReference) continue;
            var reference = Metadata.GetTypeReference((TypeReferenceHandle)type);
            var ns = Metadata.GetString(reference.Namespace); var name = Metadata.GetString(reference.Name);
            if (ns == "System.Diagnostics" && name is "DebuggerHiddenAttribute" or "DebuggerStepThroughAttribute" or "DebuggerNonUserCodeAttribute"
                || includeGenerated && ns == "System.Runtime.CompilerServices" && name == "CompilerGeneratedAttribute") return true;
        }
        return false;
    }
}
