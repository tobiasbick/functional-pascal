using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private enum RootKind { Argument, Local, AsyncField, Static }
    private sealed record ValueRoot(long Frame, RootKind Kind, uint Slot, long Module = 0, uint Type = 0, long? TypeReference = null, bool FrameType = false,
        long ContextModule = 0, uint ContextType = 0);
    private sealed record ValueHop(long Module = 0, uint Type = 0, uint Field = 0, uint? Index = null);
    private sealed record ValuePath(ValueRoot Root, ValueHop[] Hops, ulong[] Ancestors);
    private readonly Dictionary<long, ValuePath> valuePaths = [];
    private readonly Dictionary<string, long> valueKeys = [];
    private readonly Dictionary<ulong, string> objectIds = [];
    private long valueSequence;

    private long ValueReference(ValuePath path)
    {
        var key = $"{path.Root.Frame}:{path.Root.Kind}:{path.Root.Slot}:{path.Root.Module}:{path.Root.Type}:{path.Root.TypeReference}:{path.Root.FrameType}:{path.Root.ContextModule}:{path.Root.ContextType}/"
            + string.Join('/', path.Hops.Select(hop => $"{hop.Module}:{hop.Type}:{hop.Field}:{hop.Index}"));
        if (valueKeys.TryGetValue(key, out var existing)) return existing;
        if (valuePaths.Count >= 4096) throw new DebugException("limit_exceeded", "The current stop has more than 4096 value references.");
        var id = ++valueSequence; valuePaths.Add(id, path); valueKeys.Add(key, id); return id;
    }

    private NativeObject ResolveValue(ValuePath path, int recursion = 0)
    {
        if (recursion >= 16) throw new DebugException("limit_exceeded", "Static value path dependencies exceed 16 levels.");
        if (!frames.TryGetValue(path.Root.Frame, out var frame)) throw new DebugException("stale_reference", "The value's frame is no longer available.");
        using var il = frame.Frame.Query(ilFrameId);
        var current = path.Root.Kind switch
        {
            RootKind.Argument => il.Object(16, path.Root.Slot), RootKind.Local => il.Object(14, path.Root.Slot),
            RootKind.AsyncField => AsyncStorage(), RootKind.Static => StaticStorage(path.Root, frame.Frame, recursion),
            _ => throw new DebugException("unavailable", "Unsupported value root.")
        };
        try
        {
            for (var depth = 0; current.UInt32(3) == 16; depth++)
            {
                if (depth == 16) throw new DebugException("limit_exceeded", "Managed reference nesting exceeds 16 levels.");
                using var address = current.Query(referenceValueId); var contents = address.Object(10); current.Dispose(); current = contents;
            }
            foreach (var hop in path.Hops)
            {
                NativeObject next;
                if (hop.Index is { } index)
                { using var array = ArrayValue(current); next = array.Object(16, index); }
                else
                {
                    var module = modules.SingleOrDefault(module => module.Id == hop.Module) ?? throw new DebugException("unavailable", "The value's module is no longer loaded.");
                    using var owner = module.Native.Object(11, hop.Type); using var instance = ObjectValue(current);
                    next = instance.Field(owner, hop.Field);
                }
                current.Dispose(); current = next;
            }
            return current;
        }
        catch { current.Dispose(); throw; }

        NativeObject AsyncStorage()
        {
            using var receiver = il.Object(16, 0); using var instance = ObjectValue(receiver); using var owner = instance.Object(7);
            return instance.Field(owner, path.Root.Slot);
        }
    }

    private static NativeObject ArrayValue(NativeObject value)
    {
        using var reference = value.Query(referenceValueId); using var contents = reference.Object(10);
        return contents.Query(arrayValueId);
    }

    private static ulong? HeapAddress(NativeObject value)
    {
        if (value.UInt32(3) is not (14 or 18 or 20 or 28 or 29)) return null;
        using var reference = value.TryQuery(referenceValueId);
        return reference is null || reference.UInt32(7) != 0 ? null : reference.UInt64(8);
    }

    private string ObjectIdentity(ulong address)
    {
        if (objectIds.TryGetValue(address, out var existing)) return existing;
        if (objectIds.Count >= 4096) throw new DebugException("limit_exceeded", "The current stop has more than 4096 object identities.");
        var next = "object-" + (objectIds.Count + 1).ToString(System.Globalization.CultureInfo.InvariantCulture);
        objectIds.Add(address, next); return next;
    }

    private void ReleaseValues() { valuePaths.Clear(); valueKeys.Clear(); objectIds.Clear(); }
}
