using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static readonly Guid heapValue2Id = new("E3AC4D6C-9CB7-43E6-96CC-B21540E5083C");
    private sealed record AsyncInstance(string Id, NativeObject Weak);
    private readonly List<AsyncInstance> asyncInstances = [];
    private long asyncSequence;

    private static NativeObject HeapHandle(NativeObject reference, uint kind)
    {
        using var contents = reference.Object(10);
        using var heap = contents.Query(heapValue2Id);
        return heap.Object(3, kind);
    }

    private string AsyncIdentity(NativeObject receiver)
    {
        using var reference = receiver.Query(referenceValueId);
        var address = reference.UInt64(8);
        foreach (var collected in asyncInstances.Where(item => item.Weak.UInt32(7) != 0).ToArray())
        { collected.Weak.Dispose(); asyncInstances.Remove(collected); }
        var existing = asyncInstances.FirstOrDefault(item => item.Weak.UInt64(8) == address);
        if (existing is not null) return existing.Id;
        if (asyncInstances.Count >= 128) throw new DebugException("limit_exceeded", "The async identity catalog exceeds 128 live instances.");
        var next = new AsyncInstance("async-" + (++asyncSequence).ToString(System.Globalization.CultureInfo.InvariantCulture), HeapHandle(reference, 2));
        asyncInstances.Add(next); return next.Id;
    }

    private AsyncFrame? AsyncFrameFor(NativeObject frame, ModuleSymbols? metadata, uint token)
    {
        var info = metadata?.AsyncMethod(token);
        if (info is null || info.MoveNext != token) return null;
        try
        {
            using var receiver = frame.Object(16, 0);
            return new(AsyncIdentity(receiver), metadata!.MethodName(info.Kickoff), "executing");
        }
        catch (NativeOperationException exception) when (exception.Result == unchecked((int)0x80131304))
        { return new(null, metadata!.MethodName(info.Kickoff), "executing", "optimized_out"); }
    }

    private BindingPage AsyncFrameBindings(NativeObject frame, ModuleSymbols symbols, uint token, int offset, int start, int count, long? frameId = null, string view = "source")
    {
        var bindings = new List<BindingStorage>();
        if (view == "source" && frameId is { } context && symbols.AsyncMethod(token) is { } async)
        {
            var kickoff = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)async.Kickoff));
            var machine = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetDeclaringType();
            var receiver = symbols.Fields((uint)MetadataTokens.GetToken(machine)).FirstOrDefault(field => field.Name == "<>4__this");
            if (receiver.Token != 0)
            {
                var path = new ValuePath(new(context, RootKind.AsyncField, receiver.Token), [], []);
                if (symbols.SourceReceiver(async.Kickoff)) bindings.Add(new("self", () => Storage(receiver.Token), path));
                else bindings.AddRange(CaptureBindings(context, symbols, (uint)MetadataTokens.GetToken(kickoff.GetDeclaringType()), path));
            }
        }
        foreach (var field in symbols.AsyncBindings(token, offset, view))
        {
            bindings.RemoveAll(item => item.Name == field.Name);
            bindings.Add(new(field.Name, () => Storage(field.Token), frameId is { } id
                ? new(new(id, RootKind.AsyncField, field.Token), [], []) : null, Cell: field.Cell));
        }
        return ReadBindings(bindings, start, count);

        NativeObject Storage(uint field)
        {
            using var receiver = frame.Object(16, 0); using var instance = ObjectValue(receiver); using var runtimeClass = instance.Object(7);
            return instance.Field(runtimeClass, field);
        }
    }

    private void ReleaseAsyncInstances()
    {
        foreach (var instance in asyncInstances) instance.Weak.Dispose(); asyncInstances.Clear();
    }
}
