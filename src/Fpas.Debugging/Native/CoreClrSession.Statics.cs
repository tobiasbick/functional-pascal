using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Runtime.InteropServices;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    internal BindingPage StaticBindings(long frameId, string? typeName, int start, int count)
    {
        if (start < 0 || count is < 1 or > 128 || typeName?.Length > 8192) throw new DebugException("invalid_argument", "Static pages require start >= 0, count 1..128 and a bounded type name.");
        lock (gate)
        {
            try
            {
                if (!frames.TryGetValue(frameId, out var frame)) throw new DebugException("stale_reference", "Use a frame from the current stop.");
                using var type = typeName is null ? FrameOwnerType(frame.Frame) : FindStaticType(typeName);
                return ReadStatics(type, frameId, start, count, frameType: typeName is null);
            }
            catch (IOException) { throw new DebugException("unavailable", "The static type's metadata or runtime storage is unavailable."); }
        }
    }

    private NativeObject FindStaticType(string name, Action? inspect = null)
    {
        using var owner = FindStaticClass(name, inspect); return DefinitionType(owner);
    }

    private NativeObject FindStaticClass(string name, Action? inspect = null)
    {
        LoadedModule? match = null; uint token = 0; var inspected = 0;
        foreach (var module in modules)
        {
            Symbols.MetadataImage metadata;
            try { metadata = MetadataFor(module.Native); }
            catch (Exception exception) when (exception is IOException or DebugException) { continue; }
            foreach (var handle in metadata.Metadata.TypeDefinitions)
            {
                if (++inspected > 65536) throw new DebugException("limit_exceeded", "Static type lookup exceeds 65536 metadata types.");
                inspect?.Invoke();
                var candidate = (uint)MetadataTokens.GetToken(handle);
                if (metadata.TypeName(candidate) != name) continue;
                if (match is not null) throw new DebugException("ambiguous_type", "The static type name identifies more than one loaded type.");
                match = module; token = candidate;
            }
        }
        if (match is null) throw new DebugException("unavailable", "The requested static type is not present in a loaded module.");
        return match.Native.Object(11, token);
    }

    private BindingPage ReadStatics(NativeObject initial, long frameId, int start, int count, long? reference = null, bool frameType = false)
    {
        if (initial.UInt32(3) is not (17 or 18)) throw new DebugException("unavailable", "Use a declaring class or constructed object for static inspection.");
        using var contextClass = initial.Object(4); using var contextModule = contextClass.Object(3);
        var contextReceipt = modules.Single(item => item.Native.SameIdentity(contextModule.Pointer));
        var bindings = new List<BindingStorage>(); var names = new HashSet<string>(StringComparer.Ordinal);
        var type = NativeObject.Borrow(initial.Pointer);
        try
        {
            for (var depth = 0; depth < 64; depth++)
            {
                using var owner = type.Object(4); using var module = owner.Object(3); var metadata = MetadataFor(module); var token = owner.UInt32(4);
                var receipt = modules.Single(item => item.Native.SameIdentity(module.Pointer));
                foreach (var field in metadata.Fields(token))
                {
                    var definition = metadata.Metadata.GetFieldDefinition((FieldDefinitionHandle)MetadataTokens.Handle((int)field.Token));
                    if ((definition.Attributes & FieldAttributes.Static) == 0) continue;
                    if (bindings.Count == 4096) throw new DebugException("limit_exceeded", "Static inspection exceeds 4096 fields.");
                    var name = names.Add(field.Name) ? field.Name : metadata.TypeName(token) + "." + field.Name;
                    var path = new ValuePath(new(frameId, RootKind.Static, field.Token, receipt.Id, token, reference, frameType, contextReceipt.Id, contextClass.UInt32(4)), [], []);
                    bindings.Add(new(name, () => ResolveValue(path), path,
                        (definition.Attributes & FieldAttributes.Literal) != 0 ? ConstantBinding(name, metadata, definition) with { Constant = true } : null));
                }
                var next = type.OptionalObject(7);
                if (next is null) return ReadBindings(bindings, start, count);
                type.Dispose(); type = next;
            }
            throw new DebugException("limit_exceeded", "Static inspection exceeds 64 base types.");
        }
        finally { type.Dispose(); }
    }

    private unsafe NativeObject StaticStorage(ValueRoot root, NativeObject frame, int recursion)
    {
        NativeObject type;
        if (root.TypeReference is { } reference)
        {
            if (!valuePaths.TryGetValue(reference, out var path)) throw new DebugException("stale_reference", "The static type's value is no longer available.");
            using var seed = ResolveValue(path, recursion + 1); type = ExactType(seed);
        }
        else if (root.FrameType)
        {
            var module = modules.SingleOrDefault(item => item.Id == root.ContextModule) ?? throw new DebugException("unavailable", "The source static scope's module is no longer loaded.");
            using var owner = module.Native.Object(11, root.ContextType); type = DefinitionType(owner, frame);
        }
        else
        {
            var module = modules.SingleOrDefault(item => item.Id == root.Module) ?? throw new DebugException("unavailable", "The static type's module is no longer loaded.");
            using var owner = module.Native.Object(11, root.Type); type = DefinitionType(owner);
        }
        try
        {
            for (var depth = 0; depth < 64; depth++)
            {
                using var owner = type.Object(4); using var module = owner.Object(3);
                var receipt = modules.SingleOrDefault(item => item.Id == root.Module);
                if (owner.UInt32(4) == root.Type && receipt is not null && receipt.Native.SameIdentity(module.Pointer))
                {
                    nint result = 0;
                    NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, nint, nint*, int>)type.Slot(8))(type.Pointer, root.Slot, frame.Pointer, &result));
                    return new(result);
                }
                var next = type.OptionalObject(7);
                if (next is null) throw new DebugException("unavailable", "The static field's declaring type is unavailable.");
                type.Dispose(); type = next;
            }
            throw new DebugException("limit_exceeded", "Static field lookup exceeds 64 base types.");
        }
        finally { type.Dispose(); }
    }

}
