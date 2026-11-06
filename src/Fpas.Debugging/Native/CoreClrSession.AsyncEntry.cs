using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed record AsyncEntryPoint(long Module, uint Method, NativeObject Breakpoint);
    private readonly List<AsyncEntryPoint> asyncEntries = [];
    private uint? asyncEntryThread;

    private void WatchAsyncCalls(uint thread)
    {
        asyncEntryThread = thread;
        foreach (var module in modules) ArmAsyncEntries(module);
    }

    private void ArmAsyncEntries(LoadedModule module)
    {
        if (asyncEntryThread is null) return;
        var symbols = catalog.Probe(module.Path, module.Identity).Symbols;
        if (symbols is null) return;
        foreach (var method in symbols.AsyncMethods.Where(method => symbols.HasSourcePoints(method.MoveNext)))
        {
            if (asyncEntries.Any(entry => entry.Module == module.Id && entry.Method == method.MoveNext)) continue;
            if (asyncEntries.Count >= 128) throw new Core.DebugException("limit_exceeded", "An into step exceeds 128 candidate async bodies.");
            asyncEntries.Add(new(module.Id, method.MoveNext, CreateCodePoint(module.Native, method.MoveNext, 0)));
        }
    }

    private bool AsyncEntryBreakpoint(nint domain, nint thread, nint pointer)
    {
        var entry = asyncEntries.FirstOrDefault(entry => entry.Breakpoint.SameIdentity(pointer));
        if (entry is null) return false;
        using var selected = NativeObject.Borrow(thread);
        if (selected.UInt32(4) != asyncEntryThread) { ContinueCallback(domain); return true; }
        using var active = selected.Object(15); using var frame = active.Query(ilFrameId);
        if (stepper is not null) { NativeObject.Check(stepper.Call(4)); stepper.Dispose(); stepper = null; }
        ReleaseAwaitStep(); ReleaseAsyncEntry();
        StartStepper(frame, "over"); ContinueCallback(domain); return true;
    }

    private void ReleaseAsyncEntry(bool deactivate = true)
    {
        asyncEntryThread = null; Exception? failure = null;
        foreach (var entry in asyncEntries)
        {
            try { if (deactivate) NativeObject.Check(entry.Breakpoint.Call(3, 0)); }
            catch (Exception exception) { failure ??= exception; }
            finally { entry.Breakpoint.Dispose(); }
        }
        asyncEntries.Clear();
        if (failure is not null) throw new IOException("Unable to release async entry breakpoints.", failure);
    }
}
