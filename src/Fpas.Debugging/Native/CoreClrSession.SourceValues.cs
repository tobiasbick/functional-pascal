using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record SourceObject(MetadataImage Metadata, long Module, uint Type, SourceShape? Shape, string[] Properties, DebugProblem? Error = null);

    private SourceObject SourceObjectFor(NativeObject value)
    {
        using var instance = ObjectValue(value); using var owner = instance.Object(7); using var module = owner.Object(3);
        var receipt = modules.Single(item => item.Native.SameIdentity(module.Pointer)); var metadata = MetadataFor(module); var token = owner.UInt32(4);
        try
        {
            var source = new SourceObject(metadata, receipt.Id, token, metadata.Shape(token), []);
            return source with { Properties = UnmappedProperties(source) };
        }
        catch (BadImageFormatException) { return new(metadata, receipt.Id, token, null, [], new("invalid_metadata", "The source projection metadata is invalid.")); }
        catch (DebugException exception) { return new(metadata, receipt.Id, token, null, [], new(exception.Code, exception.Message)); }
    }

    private BindingPage SourceChildren(NativeObject value, ValuePath path, ulong[] ancestors, int start, int count)
    {
        var source = SourceObjectFor(value); var bindings = new List<BindingStorage>();
        if (source.Error is { } error) throw new DebugException(error.Code, error.Message);
        if (source.Shape is { } shape)
        {
            if (shape.Tag is { } tag) bindings.Add(new("Tag", () => throw new InvalidOperationException(),
                Constant: new("Tag", "int", "integer", tag.ToString(System.Globalization.CultureInfo.InvariantCulture))));
            foreach (var member in shape.Members)
            {
                var next = path with { Hops = [.. path.Hops, new(source.Module, source.Type, member.Field)], Ancestors = ancestors };
                bindings.Add(new(member.Name, () => ResolveValue(next), next,
                    Format: shape.Kind == "domain" ? value => DecodeDomain(value, shape) : null));
            }
        }
        else
        {
            foreach (var field in StoredMembers(value))
            {
                var next = path with { Hops = [.. path.Hops, new(field.Module, field.Owner, field.Token)], Ancestors = ancestors };
                bindings.Add(new(field.Name, () => ResolveValue(next), next));
            }
        }
        foreach (var name in source.Properties)
            bindings.Add(new(name, () => throw new InvalidOperationException(), Constant: new(name, "[unavailable]", "property", null, "unavailable",
                Error: new("requires_execution", "A custom property getter is outside passive inspection."))));
        return ReadBindings(bindings, start, count);
    }

    private static string[] UnmappedProperties(SourceObject source)
    {
        var definition = source.Metadata.Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)source.Type));
        if (definition.GetProperties().Count > 4096) throw new DebugException("limit_exceeded", "Source inspection exceeds 4096 properties.");
        var mapped = source.Shape?.Members.Select(member => member.Name).ToHashSet(StringComparer.Ordinal) ?? [];
        if (source.Shape?.Tag is not null) mapped.Add("Tag");
        var result = new List<string>();
        foreach (var handle in definition.GetProperties())
        {
            var property = source.Metadata.Metadata.GetPropertyDefinition(handle); var name = source.Metadata.Metadata.GetString(property.Name);
            var getter = property.GetAccessors().Getter;
            if (mapped.Contains(name) || getter.IsNil || (source.Metadata.Metadata.GetMethodDefinition(getter).Attributes & MethodAttributes.Static) != 0) continue;
            result.Add(name);
        }
        return result.ToArray();
    }
}
