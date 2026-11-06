using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Runtime.InteropServices;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record StoredMember(string Name, long Module, uint Owner, uint Token);

    private DebugBinding CompoundValue(string name, NativeObject value, string type, ValuePath? path)
    {
        var address = HeapAddress(value);
        var identity = address is { } pointer ? ObjectIdentity(pointer) : null;
        var cycle = address is { } current && path?.Ancestors.Contains(current) == true;
        int count; int[]? lengths = null, lower = null;
        var array = value.UInt32(3) is 20 or 29;
        if (array)
        {
            using var storage = ArrayValue(value);
            count = checked((int)storage.UInt32(11)); (lengths, lower) = ArrayShape(storage);
        }
        else count = StoredMembers(value).Length;
        var source = array ? null : SourceObjectFor(value);
        int? sourceCount = source?.Error is not null ? null : source is null ? count : (source.Shape?.Members.Length ?? count) + (source.Shape?.Tag is null ? 0 : 1) + source.Properties.Length;
        var clipped = path is null || path.Hops.Length >= 8;
        return new(name, type, array ? "array" : source?.Shape?.Kind ?? (value.UInt32(3) == 17 ? "struct" : "object"), source?.Shape?.Case,
            Truncated: clipped && (!array || count != 0), Reference: array && count == 0 || clipped || cycle ? null : ValueReference(path!),
            Children: count, ObjectId: identity, Cycle: cycle, Lengths: lengths, LowerBounds: lower, SourceChildren: sourceCount, SourceError: source?.Error);
    }

    internal BindingPage ValueChildren(long reference, int start, int count, string view)
    {
        if (view is not ("raw" or "source" or "static")) throw new DebugException("unsupported_view", "Use source, raw or static stored field inspection.");
        if (start < 0 || count is < 1 or > 128) throw new DebugException("invalid_argument", "Value pages require a nonnegative start and count 1..128.");
        lock (gate)
        {
            if (!valuePaths.TryGetValue(reference, out var path)) throw new DebugException("stale_reference", "Use a value reference from the current stop.");
            using var value = ResolveValue(path);
            if (view == "static")
            {
                using var type = ExactType(value);
                return ReadStatics(type, path.Root.Frame, start, count, reference);
            }
            var address = HeapAddress(value);
            var ancestors = address is { } pointer ? path.Ancestors.Append(pointer).ToArray() : path.Ancestors;
            if (value.UInt32(3) is 20 or 29)
            {
                using var array = ArrayValue(value);
                var total = checked((int)array.UInt32(11)); var (lengths, lower) = ArrayShape(array);
                var bindings = Enumerable.Range(start, start >= total ? 0 : Math.Min(count, total - start)).Select(index =>
                {
                    var next = new ValuePath(path.Root, [.. path.Hops, new(Index: (uint)index)], ancestors);
                    return new BindingStorage(ArrayLabel(index, lengths, lower), () => ResolveValue(next), next);
                });
                var page = ReadBindings(bindings, 0, count); return page with { Total = total };
            }
            if (view == "source")
            {
                try { return SourceChildren(value, path, ancestors, start, count); }
                catch (BadImageFormatException) { throw new DebugException("invalid_metadata", "The source projection metadata is invalid."); }
            }
            var fields = StoredMembers(value);
            var items = fields.Select(field =>
            {
                var next = new ValuePath(path.Root, [.. path.Hops, new(field.Module, field.Owner, field.Token)], ancestors);
                return new BindingStorage(field.Name, () => ResolveValue(next), next);
            });
            return ReadBindings(items, start, count);
        }
    }

    private StoredMember[] StoredMembers(NativeObject value)
    {
        using var instance = ObjectValue(value); var type = ExactType(instance);
        var result = new List<StoredMember>(); var names = new HashSet<string>(StringComparer.Ordinal);
        try
        {
            for (var depth = 0; depth < 64; depth++)
            {
                using var owner = depth == 0 ? instance.Object(7) : type.Object(4); using var module = owner.Object(3);
                var receipt = modules.SingleOrDefault(item => item.Native.SameIdentity(module.Pointer)) ?? throw new DebugException("unavailable", "The stored type's module is not available.");
                var metadata = MetadataFor(module); var token = owner.UInt32(4);
                foreach (var field in metadata.Fields(token))
                {
                    var definition = metadata.Metadata.GetFieldDefinition((FieldDefinitionHandle)MetadataTokens.Handle((int)field.Token));
                    if ((definition.Attributes & FieldAttributes.Static) != 0) continue;
                    if (result.Count == 4096) throw new DebugException("limit_exceeded", "A stored object has more than 4096 fields.");
                    var name = names.Add(field.Name) ? field.Name : metadata.TypeName(token) + "." + field.Name;
                    result.Add(new(name, receipt.Id, token, field.Token));
                }
                if (Values.ScalarValue.Size(type.UInt32(3)) > 0) return result.ToArray();
                var next = type.OptionalObject(7);
                if (next is null) return result.ToArray();
                type.Dispose(); type = next;
            }
            throw new DebugException("limit_exceeded", "Stored-field expansion exceeds 64 base types.");
        }
        finally { type.Dispose(); }
    }

    private static unsafe (int[] Lengths, int[] Lower) ArrayShape(NativeObject array)
    {
        var rank = array.UInt32(10);
        if (rank is < 1 or > 32) throw new DebugException("unavailable", "The array rank is outside 1..32.");
        var lengths = new int[rank]; var lower = new int[rank];
        fixed (int* dimensions = lengths)
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, int*, int>)array.Slot(12))(array.Pointer, rank, dimensions));
        if (array.UInt32(13) != 0)
            fixed (int* bounds = lower)
                NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, int*, int>)array.Slot(14))(array.Pointer, rank, bounds));
        return (lengths, lower);
    }

    private static string ArrayLabel(int index, int[] lengths, int[] lower)
    {
        var indices = new long[lengths.Length];
        for (var dimension = lengths.Length - 1; dimension >= 0; dimension--)
        {
            indices[dimension] = (long)index % lengths[dimension] + lower[dimension]; index /= lengths[dimension];
        }
        return "[" + string.Join(',', indices) + "]";
    }
}
