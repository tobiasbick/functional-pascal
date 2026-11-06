using Fpas.Debugging.Core;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    internal void BreakpointRejected(nint pointer, uint error)
    {
        lock (gate)
        {
            var rejected = points.FirstOrDefault(item => item.Value.Native.SameIdentity(pointer));
            if (rejected.Value is null) throw new IOException($"CoreCLR rejected an internal breakpoint (0x{error:X8}).");
            var loaded = modules.Single(item => item.Id == rejected.Key.Module);
            if (rejected.Value.NativeEntry) { RejectBinding(rejected.Key, rejected.Value); return; }
            using var function = loaded.Native.Object(9, rejected.Key.Method);
            using var extended = function.TryQuery(new("72965963-34fd-46e9-9434-b817fe6e7f43"));
            if (extended is null) { RejectBinding(rejected.Key, rejected.Value); return; }
            var entry = extended.Object(3);
            try { NativeObject.Check(entry.Call(3, 1)); Replace(rejected.Key, rejected.Value, entry, true); }
            catch { entry.Dispose(); throw; }
        }
    }

    private void BindAtNativeEntry(PointKey key, BoundPoint pending)
    {
        var loaded = modules.Single(item => item.Id == key.Module);
        NativeObject? replacement = null;
        try
        {
            replacement = CreateCodePoint(loaded.Native, key.Method, key.Offset);
            Replace(key, pending, replacement, false); replacement = null;
        }
        catch (NativeOperationException exception) when (exception.Result == unchecked((int)0x80131345))
        { RejectBinding(key, pending); }
        catch (DebugException exception) when (exception.Code is "unavailable" or "limit_exceeded")
        { RejectBinding(key, pending, exception.Code == "limit_exceeded" ? new("limit_exceeded", exception.Message) : null); }
        finally { replacement?.Dispose(); }
    }

    private void Replace(PointKey key, BoundPoint old, NativeObject replacement, bool entry)
    {
        NativeObject.Check(old.Native.Call(3, 0)); old.Native.Dispose();
        points[key] = old with { Native = replacement, NativeEntry = entry };
    }

    private void RejectBinding(PointKey key, BoundPoint point, DebugProblem? problem = null)
    {
        NativeObject.Check(point.Native.Call(3, 0)); point.Native.Dispose(); points.Remove(key);
        bindingErrors[key] = problem ?? new("unbound_breakpoint", "CoreCLR cannot bind this source point in the selected target."); BindBreakpoints();
    }
}
