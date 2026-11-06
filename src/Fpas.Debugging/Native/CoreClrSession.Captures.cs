using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private ValueHop CellHop(NativeObject value)
    {
        using var instance = ObjectValue(value); using var owner = instance.Object(7); using var module = owner.Object(3);
        var metadata = MetadataFor(module); var type = owner.UInt32(4);
        if (metadata.TypeName(type) != "System.Runtime.CompilerServices.StrongBox`1") throw new DebugException("unavailable", "The compiler capture cell's storage shape is unavailable.");
        var field = metadata.Fields(type).FirstOrDefault(field => field.Name == "Value");
        if (field.Token == 0) throw new DebugException("unavailable", "The compiler capture cell's value field is unavailable.");
        var receipt = modules.Single(item => item.Native.SameIdentity(module.Pointer));
        return new(receipt.Id, type, field.Token);
    }

    private IEnumerable<BindingStorage> FrameCaptures(long frame, uint token, ModuleSymbols symbols)
    {
        var method = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
        if ((method.Attributes & MethodAttributes.Static) != 0) return [];
        return CaptureBindings(frame, symbols, (uint)MetadataTokens.GetToken(method.GetDeclaringType()), new(new(frame, RootKind.Argument, 0), [], []));
    }

    private BindingStorage[] CaptureBindings(long frame, ModuleSymbols symbols, uint type, ValuePath receiver)
    {
        using var function = frames[frame].Frame.Object(5); using var module = function.Object(3);
        var receipt = modules.Single(item => item.Native.SameIdentity(module.Pointer));
        var result = new List<BindingStorage>();
        foreach (var field in symbols.Fields(type))
        {
            var storage = symbols.Storage(MetadataTokens.EntityHandle((int)field.Token));
            if (storage is null) continue;
            if (result.Count == 4096) throw new DebugException("limit_exceeded", "The closure has more than 4096 source captures.");
            var path = receiver with { Hops = [.. receiver.Hops, new ValueHop(receipt.Id, type, field.Token)] };
            result.Add(new(storage.Name, () => ResolveValue(path), path, Cell: storage.Cell));
        }
        return result.GroupBy(item => item.Name, StringComparer.Ordinal).Select(group => group.Count() == 1 ? group.First()
            : new BindingStorage(group.Key, group.First().Read, Constant: new(group.Key, "[unavailable]", "unknown", null, "unavailable",
                Error: new("ambiguous_binding", "More than one captured storage location has this source name.")))).ToArray();
    }
}
