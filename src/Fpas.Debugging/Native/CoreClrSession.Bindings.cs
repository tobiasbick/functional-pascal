using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record BindingStorage(string Name, Func<NativeObject> Read, ValuePath? Path = null, DebugBinding? Constant = null, bool Cell = false,
        Func<DebugBinding, DebugBinding>? Format = null);

    internal BindingPage FrameBindings(long frameId, int start, int count, string view)
    {
        if (view is not ("source" or "raw")) throw new DebugException("unsupported_view", "Use source or raw frame bindings.");
        if (start < 0 || count is < 1 or > 128) throw new DebugException("invalid_argument", "Binding pages require a nonnegative start and count 1..128.");
        lock (gate)
        {
            if (!frames.TryGetValue(frameId, out var frame)) throw new DebugException("stale_reference", "Use a frame from the current stop.");
            if (frame.Symbols is null || frame.Data.IlOffset is not { } offset) throw new DebugException("unavailable", "No lexical context is available for this frame.");
            using var il = frame.Frame.Query(ilFrameId);
            if (frame.Data.Async is not null) return AsyncFrameBindings(il, frame.Symbols, frame.Data.MethodToken, offset, start, count, frameId, view);
            var bindings = view == "source" ? FrameCaptures(frameId, frame.Data.MethodToken, frame.Symbols).ToList() : [];
            foreach (var argument in frame.Symbols.Arguments(frame.Data.MethodToken, view))
            {
                bindings.RemoveAll(item => item.Name == argument.Name);
                bindings.Add(new(argument.Name, () => il.Object(16, (uint)argument.Index), new(new(frameId, RootKind.Argument, (uint)argument.Index), [], [])));
            }
            foreach (var local in frame.Symbols.Locals(frame.Data.MethodToken, offset))
            {
                bindings.RemoveAll(item => item.Name == local.Name);
                bindings.Add(new(local.Name, () => il.Object(14, (uint)local.Slot), new(new(frameId, RootKind.Local, (uint)local.Slot), [], []), Cell: view == "source" && local.Cell));
            }
            return ReadBindings(bindings, start, count);
        }
    }

    private BindingPage ReadBindings(IEnumerable<BindingStorage> bindings, int start, int count)
    {
        var all = bindings.Take(4097).ToArray();
        if (all.Length > 4096) throw new DebugException("limit_exceeded", "A frame has more than 4096 visible bindings.");
        var result = new List<DebugBinding>(); var budget = 512 * 1024;
        foreach (var item in all.Skip(start).Take(count))
        {
            var value = item.Constant ?? ReadBinding(item.Name, item.Read, item.Path, item.Cell);
            if (item.Format is not null) value = item.Format(value);
            var bound = BindingBound(value);
            if (bound > 512 * 1024)
            {
                value = new(value.Name, value.Type, value.Kind, null, "unavailable", true,
                    Error: new("limit_exceeded", "The value description exceeds one inspection page."));
                bound = BindingBound(value);
            }
            if (bound > budget) return new(result.ToArray(), all.Length, true);
            budget -= (int)bound; result.Add(value);
        }
        return new(result.ToArray(), all.Length);
    }

    private static long BindingBound(DebugBinding value) => 640L + 6L * (value.Name.Length + value.Type.Length
        + (value.Scalar?.Length ?? 0) + (value.RawUtf16?.Length ?? 0) + (value.Error?.Code.Length ?? 0) + (value.Error?.Message.Length ?? 0)
        + (value.SourceError?.Code.Length ?? 0) + (value.SourceError?.Message.Length ?? 0)) + TypeBound(value.RuntimeType);

    private static long TypeBound(DebugTypeIdentity? type) => type is null ? 0 : 640L + 6L * type.Name.Length
        + (type.Arguments?.Sum(TypeBound) ?? 0);
}
